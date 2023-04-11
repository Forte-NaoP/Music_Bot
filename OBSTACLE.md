# 문제점들

> 1. 음성채널에서 음악을 재생하고 싶음
>     1. url로 부터 음악을 재생하고 싶음
>           - ytdl 사용
>     2. 큐에 넣고 재생하고 싶음
>           - ~~`Arc<RwLock<VecDeque>>` 사용~~
>           - bot의 client에 저장되는 데이터 타입 `GuildQueue` 구조 고민중
>               - 여러 서버에서 접근하는 경우를 생각하면 client에 저장될 `GuildQueueContainer`는 `Arc<RwLock<HashMap<GuildId, GuildQueue>>>` 이어야 할 듯.
>               - 그러면 `GuildQueue` 내부 구조는?
>                   - 생각하는 시나리오는 재생관련 명령어 실행하면 `GuildQueue` 내부의 url vector에서 url pop해서<br>`Songbird::input::Input` 으로 변환 후 `Call::enqueue_source`하는 건데,<br>url_vector에서 enqueue_source까지의 과정을 여러개를 동시에 하고 싶음.<br>아니면 적어도 노래 재생되는 동안 하나씩이라도 넣어서 최대한 노래 중간중간 안 끊기게 하고 싶음.
>                   - 다른 의견으로는 아예 백엔드 서버가 있음.
>                   - 우선은 하나씩 하게 하는걸로
>     3. 곡의 시작시간이나 재생시간을 조정하고 싶음
>           - sorgbird 에서는 지원 안하므로 songbird ytdl을 적당히 수정할 계획
>               - 기존 ytdl은 재생되기 전까지 yt-dlp와 ffmpeg 프로세스를 잡고 있는 듯함. (아닐수도 있음)
>               - 우선 yt-dlp는 파일을 저장하고 ffmpeg에서 파일을 읽어서 오디오 처리하도록 함.
>                   - 네트워크 -> 메모리 -> 디스크 -> 메모리 경로 때문에 기존보다 느려질 수는 있으나<br>같은 곡이 여러번 재생 될 경우를 생각해서 유튜브 영상은 저장해두기로 함.
>                   - pipe에 대해 헷갈리는 부분이 있어서 고생했음.
>     4. 곡의 재생에 맞춰 임베드로 타이머를 업데이트 하려고 함
>           - 타이머 업데이트 타이밍이 잘 맞지 않음.
>     5. 봇을 테스트 하는 중에 Driver 에러가 계속해서 발생함. 서버에서 봇을 추방했다 다시 초대하니 해결됐지만<br>다른 해결방법을 생각해봐야 할 듯 함.

---

> 2. DB에서 url을 관리하고 싶음
>       1. (tokio)rusqlite 사용
>       2. url과 title을 1:N 관계로 묶고 싶음
>               - url과 title 테이블 분리 및 url_title 연결 테이블 생성
