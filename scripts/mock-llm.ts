const port = Number(process.env.PORT ?? 8787);
const enc = new TextEncoder();
let calls = 0;

Bun.serve({
  port,
  fetch(req) {
    const url = new URL(req.url);
    if (url.pathname.endsWith('/chat/completions')) {
      calls++;
      const stream = new ReadableStream({
        start(ctrl) {
          const send = (obj: unknown) =>
            ctrl.enqueue(enc.encode(`data: ${JSON.stringify(obj)}\n\n`));
          if (calls === 1) {
            send({ choices: [{ delta: { content: '你好，' } }] });
            send({ choices: [{ delta: { content: '我是 Zeno。' } }] });
            send({
              choices: [
                {
                  delta: {
                    tool_calls: [
                      {
                        index: 0,
                        function: { name: 'read_file', arguments: '{"path":"README.md"}' }
                      }
                    ]
                  }
                }
              ]
            });
            send({ choices: [{ finish_reason: 'tool_calls' }] });
          } else {
            send({
              choices: [{ delta: { content: '已读取 README.md，这是 Zeno 项目的说明。' } }]
            });
            send({ choices: [{ finish_reason: 'stop' }] });
          }
          ctrl.enqueue(enc.encode('data: [DONE]\n\n'));
          ctrl.close();
        }
      });
      return new Response(stream, { headers: { 'content-type': 'text/event-stream' } });
    }
    return new Response('not found', { status: 404 });
  }
});

console.log(`mock LLM server on http://localhost:${port}`);
