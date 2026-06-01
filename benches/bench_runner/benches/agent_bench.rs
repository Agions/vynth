use criterion::{black_box, criterion_group, criterion_main, Criterion};
use synerix::agent::context::estimate_tokens;
use synerix::agent::context::{ContextManager, TokenBudget};
use synerix::llm::types::ChatMessage;

fn bench_estimate_tokens(c: &mut Criterion) {
    let short_text = "Hello, world!";
    let chinese_text = "你好世界，这是一个中文测试文本，用于评估标记估算性能。";
    let code_text = r#"
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci() {
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);
        assert_eq!(fibonacci(10), 55);
    }
}
"#;

    c.bench_function("estimate_tokens/short_text", |b| {
        b.iter(|| estimate_tokens(black_box(short_text)))
    });

    c.bench_function("estimate_tokens/chinese_text", |b| {
        b.iter(|| estimate_tokens(black_box(chinese_text)))
    });

    c.bench_function("estimate_tokens/code_text", |b| {
        b.iter(|| estimate_tokens(black_box(code_text)))
    });
}

fn bench_context_push_and_trim(c: &mut Criterion) {
    c.bench_function("context/push_100_messages", |b| {
        b.iter(|| {
            let budget = TokenBudget::new(black_box(1000));
            let mut ctx = ContextManager::new(budget);

            for i in 0..100 {
                ctx.push(ChatMessage::user(&format!("Message {} with some extra content to consume token budget so trimming is triggered", i)));
            }

            black_box(ctx.messages().len());
        })
    });
}

criterion_group!(benches, bench_estimate_tokens, bench_context_push_and_trim);
criterion_main!(benches);
