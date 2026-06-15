use std::sync::Arc;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use synerix::tools::builtin::*;
use synerix::tools::registry::ToolRegistry;

fn bench_collect_tool_schemas(c: &mut Criterion) {
    // Build a realistic tool registry
    let mut tools = ToolRegistry::new();
    tools.register(Arc::new(FileReadTool));
    tools.register(Arc::new(FileWriteTool));
    tools.register(Arc::new(PatchTool));
    tools.register(Arc::new(SearchTool));
    tools.register(Arc::new(ShellExecTool));

    c.bench_function("agent/collect_schemas_5_tools", |b| {
        b.iter(|| {
            let schemas = tools.all_schemas();
            black_box(schemas.len());
        })
    });
}

criterion_group!(agent_benches, bench_collect_tool_schemas);
criterion_main!(agent_benches);
