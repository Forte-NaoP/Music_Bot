# 지식이 늘었다

- `std::process::Stdio::piped()` 는 새로운 파이프를 생성한다. `std::process::Command::stdin(Stdio::piped())` 같은 식으로 사용하면 stdin이 생성된 파이프로 대체 되는 식.<br>
`CommandObject.stdin.take().unwrap()`으로 해당 파이프 디스크립터를 가져올 수 있다.

- `Command::spawn()`으로 생성된 `Child` 객체는 `wait[_with_output]`나 `kill` 호출 전까지 완료/종료되지 않는다.<br>
만약 파일을 저장하는 작업 이후 연속해서 다른 `Command` 가 그 파일을 읽으려 한다면 파일이 생성되지 않아 정상적으로 작동하지 않는다. 버퍼 관련 문제인듯<br>
따라서 `wait` 를 사용해 프로세스를 기다리면 파일이 저장되어 사용 가능하다.<br>
하지만 pipe로 출력과 입력을 연결하면 출력버퍼에서 직접 스트림을 읽을 수 있기 때문에 wait가 필요 없다.

- 파이프 관련하여<br><br>
    만약 `Command::spawn()`으로 프로세스를 하나 생성 할 때, 이 프로세스의 `stdin`과 `stdout`을 모두 파이프 처리했을 때 deadlock이 발생할 수 있다.<br>예를 들어
    ```rust
        use std::{
            io::{Read, Write},
            process::{Command, Stdio},
        };

        let mut ffmpeg = Command::new("ffmpeg")
            .args(&["-i", "-"])
            .args(&FFMPEG_ARGS)
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let mut stdin = ffmpeg.stdin.take().unwrap();

        stdin.write_all(&data).unwrap();
        
        let mut output = vec![];
        let mut stdout = ffmpeg.stdout.take().unwrap();
        stdout.read_to_end(&mut output).unwrap();
        
        ffmpeg.wait().unwrap();
    ```
    다음과 같은 코드가 있을 때, `stdin.write_all(&data).unwrap();`부분에서 deadlock이 발생할 수 있다.<br>가능한 시나리오는 다음과 같다.<br>
    1. write_all 함수를 통해 프로세스(이하 A)의 stdin에 데이터를 쓴다.
    2. A는 stdin에서 데이터를 소비하여 stdout에 데이터를 쓴다.
    3. 이때 데이터를 소비하는 속도보다 stdin에 데이터를 쓰는 속도가 빨라 stdin이 가득 차게 되면 write_all 함수는 block 된다.
    4. A가 작업을 진행하여 stdout가 가득 차게 되면 A도 block된다. 
    5. A가 stdin에서 데이터를 소비할 수 없게 되어 write_all은 계속 블록된 상태로 있고, `stdout.read_to_end(&mut output).unwrap();`로 진행할 수 없게 되어 A의 stdout에서 데이터를 가져와 stdout을 비울 수 없게 된다.
    6. 따라서 A도 계속 block된 상태로 남아있게 되므로 deadlock이 발생한다.

    이를 해결하려면 
    `stdin.write_all(&data).unwrap();`를 `tokio::spawn`으로 감싸 task를 분리해주면 된다.
    ```rust
    tokio::spawn(async move {
        stdin.write_all(&data).await.unwrap();
    });
    ```
    이렇게 하면 `write_all`은 여전히 block되지만 다른 task에서 block되므로 기존의 task는 `stdout.read_to_end(&mut output).unwrap();`까지 도달하여 stdout을 소비하여 spawn된 프로세스를 계속 실행시킬 수 있기 때문에 deadlock이 발생하지 않게 된다.
    
    
    


- `Arc` 타입에 관해
    - 원자적으로 동작하는 복수의 소유권을 가지는 스마트 포인터
    - `Arc::clone` 하면 값을 clone하는게 아니라 소유권을 가져오면서 참조 카운팅을 늘린다.
    - 단 해당 참조는 `immutable`하므로 `Mutex`나 `RwLock`를 감싸서 `Arc<Mutex<T>>`로 가변성을 얻는 식으로 많이 사용한다.
