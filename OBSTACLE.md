# 문제점들
> 1. 음성채널에서 음악을 재생하고 싶음
>     1. url로 부터 음악을 재생하고 싶음
>           - ytdl 사용
>     2. 큐에 넣고 재생하고 싶음
>           - `Arc<RwLock<VecDeque>>` 사용
>     3. 곡의 시작시간이나 재생시간을 조정하고 싶음
>           - 현재 진행중 <br>sorgbird 에서는 지원 안하므로 songbird Input을 모사해서<br>새로운 structure 만들어야함
---
> 2. DB에서 url을 관리하고 싶음
>       1. (tokio)rusqlite 사용
>       2. url과 title을 1:N 관계로 묶고 싶음
>               - url과 title 테이블 분리 및 url_title 연결 테이블 생성