# 지식이 늘었다

- `std::process::Stdio::piped()` 는 새로운 파이프를 생성한다. `std::process::Command::stdin(Stdio::piped())` 같은 식으로 사용하면 stdin이 생성된 파이프로 대체 되는 식.<br>
`CommandObject.stdin.take().unwrap()`으로 해당 파이프 디스크립터를 가져올 수 있다.

- `Command::spawn()`으로 생성된 `Child` 객체는 `wait[_with_output]`나 `kill` 호출 전까지 완료/종료되지 않는다.<br>
만약 파일을 저장하는 작업 이후 연속해서 다른 `Command` 가 그 파일을 읽으려 한다면 파일이 생성되지 않아 정상적으로 작동하지 않는다. 버퍼 관련 문제인듯<br>
따라서 `wait` 를 사용해 프로세스를 기다리면 파일이 저장되어 사용 가능하다.<br>
하지만 pipe로 출력과 입력을 연결하면 출력버퍼에서 직접 스트림을 읽을 수 있기 때문에 wait가 필요 없다.

- `Arc` 타입에 관해
    - 원자적으로 동작하는 복수의 소유권을 가지는 스마트 포인터
    - `Arc::clone` 하면 값을 clone하는게 아니라 소유권을 가져오면서 참조 카운팅을 늘린다.
    - 단 해당 참조는 `immutable`하므로 `Mutex`나 `RwLock`를 감싸서 `Arc<Mutex<T>>`로 가변성을 얻는 식으로 많이 사용한다.
