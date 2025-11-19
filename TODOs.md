# TODOs

[ ] history添加不同的sink
[ ] ws agent能看到fyi message
[ ] 去掉所有的unwrap
[ ] use ai to gen commit msg
[ ] large files: src/history.rs - 1,176 lines (refactor into modules)
[ ] large files: src/game.rs - 783 lines (split by responsibility)
[ ] Player iteration inefficiencies: PlayerIndexedVec is present, but some spots iterate over indices instead of using it directly.
[ ] 53 println! calls; use tracing consistently.
[ ] WebSocket handlers spawn without awaiting joins. // TODO: join handle_connection
[ ] detect typos automaticly
[ ] 66 total cards, 8 buildings, Extract to constants or enums
[ ] use websocket for agent communications
[ ] room管理
