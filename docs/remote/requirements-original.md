# Mdir4 SSH Remote / Remote Drive 요구사항

## 1. 개요

### 1.1 목적

Mdir4에서 SSH로 연결된 원격 호스트의 파일 시스템을 로컬 드라이브와 유사한 방식으로 탐색하고 관리할 수 있도록 한다.

Remote 기능은 별도의 FTP/SFTP 클라이언트 UI를 제공하는 방식이 아니라, Mdir III의 `A:`, `B:`, `C:`, `D:` 드라이브 전환 경험을 확장한 **Location / Remote Drive** 개념으로 구현한다.

사용자는 다음 대상을 가능한 한 동일한 방식으로 탐색해야 한다.

```text
C:
D:
E:
DEV
PROD
NAS
```

예:

```text
C:      Local Drive
D:      Local Drive
DEV     SSH Remote
PROD    SSH Remote
NAS     SSH Remote
```

---

# 2. 핵심 원칙

SSH Remote는 다음 원칙을 따른다.

1. SSH 인증 정보를 Mdir4가 직접 관리하지 않는다.
2. SSH 연결 정보는 OpenSSH 설정을 우선 사용한다.
3. 사용자명과 비밀번호 입력 UI를 제공하지 않는다.
4. Private Key 자체를 Mdir4 설정에 저장하지 않는다.
5. Remote는 Local Drive와 동일한 Location 개념으로 취급한다.
6. 파일 목록 UI는 Local과 Remote에서 최대한 동일하게 동작한다.
7. 네트워크 지연 때문에 UI가 정지해서는 안 된다.
8. 연결 실패가 파일 관리자 전체에 영향을 주어서는 안 된다.

---

# 3. 인증 정책

## 3.1 OpenSSH 설정 사용

기본 SSH 설정은 다음 파일을 사용한다.

```text
~/.ssh/config
```

Windows에서는 사용자의 OpenSSH 설정 경로를 따른다.

Mdir4는 다음과 같은 SSH Host alias를 사용한다.

```ssh
Host dev
    HostName 10.0.0.20
    User ubuntu
    IdentityFile ~/.ssh/id_ed25519

Host production
    HostName prod.example.com
    User deploy
    IdentityFile ~/.ssh/prod_ed25519
    ProxyJump bastion
```

Mdir4에서 사용하는 값은:

```text
dev
production
```

뿐이다.

---

# 4. Mdir4에서 관리하지 않는 인증 정보

Mdir4 자체 설정에서는 다음 정보를 저장하거나 입력받지 않는다.

```text
Username
Password
Private Key
Private Key Password
Port
HostName
ProxyJump
ProxyCommand
```

이 정보는 OpenSSH 설정의 책임으로 한다.

즉 다음과 같은 Mdir4 설정은 허용하지 않는다.

```toml
username = "ubuntu"
password = "secret"
private_key = "/path/key"
```

---

# 5. Password 인증

Mdir4에서는 대화형 Password 입력을 지원하지 않는다.

연결 과정에서 다음 입력창을 표시하지 않는다.

```text
Username:
Password:
```

SSH 인증이 자동으로 완료되지 않으면 연결 실패로 처리한다.

예:

```text
SSH authentication failed.

Configure this host in ~/.ssh/config.
```

Mdir4가 인증 비밀번호 또는 자격 증명을 저장하는 기능도 제공하지 않는다.

---

# 6. SSH Agent

SSH Agent 기반 인증은 허용한다.

사용자가 기존 OpenSSH 환경에서:

```text
ssh dev
```

명령으로 별도 Password 입력 없이 정상 연결할 수 있다면 Mdir4에서도 가능한 한 동일한 환경을 사용한다.

---

# 7. Host Key 검증

SSH Host Key 검증을 비활성화하지 않는다.

다음 보안 기능을 기존 OpenSSH 환경과 동일하게 사용한다.

```text
known_hosts
Host Key Verification
SSH Config
SSH Agent
```

Mdir4 편의를 위해 Host Key 검증을 무조건 우회하는 옵션은 제공하지 않는다.

---

# 8. Remote 설정

Mdir4는 인증 정보가 아니라 **Remote Location 정의**만 저장한다.

예:

```toml
[[remote]]
name = "DEV"
host = "dev"
root = "/home/ubuntu"

[[remote]]
name = "PROD"
host = "production"
root = "/var/www"

[[remote]]
name = "NAS"
host = "nas"
root = "/volume1"
```

각 필드의 의미:

```text
name
Mdir4에서 표시할 이름

host
~/.ssh/config의 Host alias

root
Remote 선택 시 최초 진입 경로
```

---

# 9. Remote ID

Remote에는 사람이 빠르게 식별할 수 있는 짧은 이름을 사용한다.

권장:

```text
DEV
PROD
NAS
WEB
DB
HOME
```

UI에서는 전체 hostname보다 이 이름을 우선 표시한다.

예:

```text
DEV:/home/ubuntu/project
```

전체 연결 문자열:

```text
ssh://ubuntu@10.0.0.20:22/home/ubuntu/project
```

은 기본 화면에 표시하지 않는다.

---

# 10. Read Only Remote

Remote에 읽기 전용 옵션을 제공한다.

예:

```toml
[[remote]]
name = "PROD"
host = "production"
root = "/var/www"
read_only = true
```

읽기 전용 Remote에서는 다음 작업을 차단한다.

```text
Rename
Write
Upload
Move
MkDir
Delete
Edit Save
```

허용:

```text
Browse
View
Download
Copy to Local
Stat
```

UI에서 다음처럼 표시한다.

```text
PROD [RO]
```

---

# 11. Location 개념

Core 내부에서는 Windows의 `Drive` 개념 대신 보다 일반적인 `Location` 개념을 사용한다.

개념 구조:

```text
Location
 ├── Local
 └── Remote
```

예:

```text
Location Manager

C:
 └── LocalFileSystem

D:
 └── LocalFileSystem

DEV
 └── SftpFileSystem

PROD
 └── SftpFileSystem

NAS
 └── SftpFileSystem
```

이를 통해 UI는 Local/Remote 여부와 최대한 독립적으로 동작한다.

---

# 12. Location 선택 화면

Mdir III의 드라이브 선택과 유사한 UX를 제공한다.

예:

```text
┌──────────────── Locations ────────────────┐
│                                           │
│ Local                                     │
│                                           │
│ C:      System                            │
│ D:      Data                              │
│ E:      USB                               │
│                                           │
│ Remote                                    │
│                                           │
│ DEV     Development                       │
│ PROD    Production                  [RO]  │
│ NAS     Storage                           │
│                                           │
└───────────────────────────────────────────┘
```

조작:

```text
↑ ↓
Location 선택

Enter
Location 진입

Esc
취소
```

---

# 13. Local과 Remote의 UX 통일

Local:

```text
C:\PROJECT

README.md       src             Cargo.toml
LICENSE         target          Cargo.lock
```

Remote:

```text
DEV:/home/ubuntu/project

README.md       src             Cargo.toml
LICENSE         target          Cargo.lock
```

파일 목록의 기본 조작은 동일해야 한다.

```text
↑ ↓ ← →
Enter
Backspace
Home
End
PgUp
PgDn
Space
Insert
```

---

# 14. Remote FileSystem

기존 FileSystem 계층을 확장한다.

```text
FileSystem
 ├── LocalFileSystem
 ├── RemoteFileSystem
 │      └── SftpFileSystem
 └── TestFileSystem
```

UI와 Navigation은 특정 FileSystem 구현을 직접 알지 않는다.

---

# 15. FileSystem API

FileSystem API는 처음부터 Remote 환경의 지연을 고려한다.

개념적으로 다음 기능을 제공한다.

```text
read_dir
stat
read
write
rename
mkdir
remove_file
remove_dir
copy
```

Remote 구현에서는 가능한 한 비동기 방식으로 처리한다.

---

# 16. Remote 기본 기능

SSH Remote의 1차 지원 기능:

```text
Directory Listing
Directory Navigation
File Stat
File View
File Download
File Upload
Rename
Move
MkDir
Delete
```

Local에서 이미 제공되는 파일 관리 기능과 최대한 동일한 UX를 사용한다.

---

# 17. Local → Remote Copy

Local 파일을 Remote로 복사할 수 있어야 한다.

예:

```text
C:\BUILD\release.zip

F5 Copy

↓

DEV:/home/ubuntu/releases/release.zip
```

Remote 업로드 작업으로 처리한다.

---

# 18. Remote → Local Copy

Remote 파일을 Local로 복사할 수 있어야 한다.

예:

```text
PROD:/var/log/app.log

F5 Copy

↓

C:\logs\app.log
```

Remote 다운로드 작업으로 처리한다.

---

# 19. Remote → Remote Copy

동일 Remote 내부의 복사/이동은 지원한다.

예:

```text
DEV:/home/user/a.txt

↓

DEV:/home/user/backup/a.txt
```

서로 다른 Remote 사이:

```text
DEV → PROD
```

복사는 초기 버전의 필수 기능으로 하지 않는다.

향후 구현 시 Local relay 또는 서버 간 전송 방식을 별도로 정의한다.

---

# 20. Remote 상태 표시

상단 경로 영역에서 Remote라는 사실을 확인할 수 있어야 한다.

예:

```text
DEV [SSH] /home/ubuntu/project
```

또는:

```text
DEV:/home/ubuntu/project
```

기본 표시에서는 후자의 간결한 표현을 권장한다.

---

# 21. 하단 상태창

Remote에서는 Local Disk Free Space 대신 Remote 상태 정보를 표시할 수 있다.

예:

```text
DEV | SSH | 24 Files | 6 Dirs | Selected 2 / 14 MB
```

연결 품질이나 latency는 초기 필수 기능으로 하지 않는다.

---

# 22. 연결 과정

Remote 선택:

```text
DEV
```

↓

```text
Connecting DEV...
```

↓

성공:

```text
DEV:/home/ubuntu
```

실패:

```text
Unable to connect to DEV.
```

파일 목록 UI는 연결 작업 동안 block되지 않아야 한다.

---

# 23. 비동기 처리

Remote FileSystem 작업은 UI thread에서 직접 수행하지 않는다.

구조:

```text
UI
 │
 ▼
Remote Request
 │
 ▼
Background Worker
 │
 ▼
SSH / SFTP
 │
 ▼
Remote Result
 │
 ▼
App Event
 │
 ▼
UI Update
```

특히 다음 작업은 반드시 비동기로 처리한다.

```text
Connect
Directory Listing
Stat
Upload
Download
Delete
Rename
```

---

# 24. Remote Cache

Remote 디렉터리 목록을 임시 캐시할 수 있다.

예:

```text
RemoteDirectoryCache {
    location
    path
    entries
    loaded_at
}
```

목적:

* 불필요한 네트워크 요청 감소
* 디렉터리 재방문 속도 개선
* 느린 연결에서 UI 응답성 유지

수동 Refresh는 항상 지원한다.

```text
Ctrl+R
```

---

# 25. 연결 유지

Remote에 접속한 동안 SSH 연결을 가능한 한 재사용한다.

다음 동작마다 새로운 SSH connection을 생성하지 않는다.

```text
Enter directory
View file
Stat
Rename
```

개념:

```text
Remote Session
    │
    ├── SFTP
    ├── Request
    ├── Request
    └── Request
```

---

# 26. 연결 끊김 처리

연결이 끊어진 경우 파일 관리자 전체를 종료하지 않는다.

예:

```text
DEV connection lost.

Enter : Reconnect
Esc   : Return
```

가능하면 마지막 캐시된 목록을 유지한다.

캐시된 화면이라는 사실은 상태창에 표시할 수 있다.

```text
DEV [Disconnected]
```

---

# 27. Remote 자동 재연결

초기 버전에서는 자동 무한 재연결을 하지 않는다.

연결 실패 시 사용자 명령으로 다시 연결하는 것을 기본으로 한다.

향후 제한된 자동 재시도 정책을 추가할 수 있다.

---

# 28. `.ssh/config` Host 검색

Mdir4는 OpenSSH config의 Host alias를 읽어 Remote 후보를 보여줄 수 있다.

예:

```text
SSH Config

dev
production
nas
bastion
github.com
test-server
```

하지만 `.ssh/config`의 모든 Host를 Remote Drive로 자동 등록하지 않는다.

---

# 29. Registered Remote와 SSH Host 구분

Location 화면에서는 Mdir4에 등록된 Remote를 우선 표시한다.

예:

```text
Remote

★ DEV
★ PROD
★ NAS
```

별도 화면 또는 하위 영역에서 SSH config Host를 보여줄 수 있다.

```text
SSH Hosts

  bastion
  github.com
  test-server
```

등록된 Remote만 기본 Location 목록에 노출하는 것을 권장한다.

---

# 30. Remote 등록

사용자는 `.ssh/config` Host를 선택하여 Mdir4 Remote로 등록할 수 있다.

필요 입력:

```text
Name
SSH Host Alias
Root Path
Read Only
```

인증 정보 입력은 없다.

예:

```text
Name      : DEV
Host      : dev
Root      : /home/ubuntu
Read Only : No
```

---

# 31. 설정 파일 예시

```toml
[remote]

[[remote.locations]]
name = "DEV"
host = "dev"
root = "/home/ubuntu"
read_only = false

[[remote.locations]]
name = "PROD"
host = "production"
root = "/var/www"
read_only = true

[[remote.locations]]
name = "NAS"
host = "nas"
root = "/volume1"
read_only = false
```

---

# 32. Remote 삭제

Mdir4에서 Remote 정의를 삭제하더라도:

```text
~/.ssh/config
known_hosts
SSH Keys
```

에는 영향을 주지 않는다.

삭제되는 것은 Mdir4 Location 등록 정보뿐이다.

---

# 33. SSH Shell

SSH Shell 기능은 Remote FileSystem과 별개 기능으로 취급한다.

구조:

```text
Remote Location
 ├── SFTP FileSystem
 └── SSH Terminal
```

초기 Remote Drive 구현의 필수 기능에는 포함하지 않는다.

향후 현재 Remote에 대해 터미널을 열 수 있다.

예:

```text
Alt+T
```

↓

```text
SSH Terminal: DEV
```

---

# 34. Remote Git

Remote Host의 Git Repository 탐지는 Remote Drive의 기본 범위에 포함하지 않는다.

향후 Git Plugin과 Remote Plugin을 연동하여:

```text
DEV:/home/user/project
```

에서 Git 상태를 표시할 수 있다.

구조:

```text
Remote FileSystem
      │
      └── Git Plugin
              │
              └── Remote Git Backend
```

별도 확장 단계로 설계한다.

---

# 35. 테스트 구조

Remote 기능 역시 실제 서버 없이 자동 테스트할 수 있어야 한다.

구조:

```text
FileSystem
 ├── LocalFileSystem
 ├── SftpFileSystem
 └── FakeRemoteFileSystem
```

`FakeRemoteFileSystem`을 사용해:

```text
Connect
Directory Listing
Navigation
Upload
Download
Rename
Delete
Disconnect
Reconnect
```

상태를 시뮬레이션한다.

---

# 36. Remote Snapshot 테스트

예:

```yaml
name: remote-navigation

terminal:
  width: 80
  height: 25

location:
  type: remote
  name: DEV

steps:
  - connect
  - snapshot: remote-root

  - key: DOWN
  - key: ENTER
  - snapshot: remote-directory

  - disconnect
  - snapshot: disconnected
```

실제 SSH 서버에 접속하지 않아야 한다.

---

# 37. 연결 지연 테스트

Fake Remote Backend에서 지연 상태를 시뮬레이션한다.

예:

```text
Connecting
Loading Directory
Transfer Running
Connection Lost
```

각 상태에서도 UI 입력 및 렌더링이 정상 동작하는지 확인한다.

---

# 38. Read Only 테스트

`read_only = true` Remote에서 다음 작업을 차단하는지 자동 테스트한다.

```text
F2 Rename
F4 Save
F6 Move
F7 MkDir
F8 Delete
Local → Remote Copy
```

허용:

```text
F3 View
Remote → Local Copy
Navigation
```

---

# 39. Phase 1

Remote Drive 1차 구현:

```text
Location Manager
Remote 설정
OpenSSH Host Alias 기반 연결
Password UI 없음
Remote FileSystem abstraction
SFTP Directory Listing
Navigation
File Stat
F3 View
Remote → Local Download
Local → Remote Upload
Rename
MkDir
Delete
Read Only Remote
Async Worker
Connection State
Fake Remote Backend
Snapshot Test
```

---

# 40. Phase 2

추가:

```text
Connection Cache
Directory Cache
Transfer Progress
Resume
Large File Handling
SSH Host Browser
Remote 등록 UI
Remote 설정 편집
Symlink 강화
File Permission 표시
```

---

# 41. Phase 3

후속 기능:

```text
SSH Terminal
Remote Git
Remote Search
Remote Archive
Remote ↔ Remote Copy
Connection Multiplexing
Advanced Proxy Support
```

---

# 42. 기술적 핵심 구조

최종적으로 Core는 Local인지 SSH인지 최대한 알지 않는다.

```text
                    Mdir4 Core
                        │
                        ▼
                 Location Manager
                        │
            ┌───────────┴───────────┐
            │                       │
            ▼                       ▼
        Local C:                  DEV
            │                       │
            ▼                       ▼
   LocalFileSystem           SftpFileSystem
                                    │
                                    ▼
                                  SSH
                                    │
                                    ▼
                              Remote Host
```

UI에는 두 Location 모두 동일하게:

```text
read_dir
stat
open
copy
rename
mkdir
remove
```

형태의 기능으로 노출한다.

---

# 43. UX 원칙

Remote는 별도의 애플리케이션처럼 보여서는 안 된다.

Mdir4 사용자에게 Remote는 단순히 또 하나의 드라이브처럼 느껴져야 한다.

즉:

```text
C:
 ↓
D:
 ↓
DEV
 ↓
PROD
```

의 전환 경험을 제공한다.

핵심 UX는:

```text
Location 선택
      ↓
Enter
      ↓
파일 탐색
```

이다.

SSH 연결 과정과 인증 구현의 복잡성은 가능한 한 사용자에게 노출하지 않는다.

---

# 44. 보안 원칙

Mdir4의 Remote 기능은 다음 원칙을 지킨다.

```text
No Password Storage
No Username/Password Dialog
No Private Key Storage
No Host-Key Verification Bypass
Use Existing OpenSSH Configuration
Use Existing SSH Agent Where Available
```

즉 Mdir4는 SSH Credential Manager가 아니다.

사용자가 이미 구축한 OpenSSH 환경 위에서 동작하는 Remote File Manager 역할만 수행한다.

---

# 45. 완료 기준

Remote Drive MVP는 다음 조건을 충족하면 완료로 본다.

* Mdir4 설정에 Remote Location 등록 가능
* `.ssh/config`의 Host alias 사용
* Mdir4에서 사용자명 입력 없음
* Mdir4에서 Password 입력 없음
* Mdir4에 Private Key 저장 없음
* Local Drive와 Remote가 같은 Location 화면에 표시됨
* Remote 디렉터리 탐색 가능
* 상하좌우 멀티 컬럼 탐색 정상
* F3 Remote File View 가능
* Local ↔ Remote 파일 복사 가능
* Remote Rename 가능
* Remote MkDir 가능
* Remote Delete 가능
* Read Only Remote 지원
* 네트워크 작업 중 UI freeze 없음
* 연결 실패 시 Core 정상 유지
* 연결 끊김 처리 가능
* Fake Remote Backend로 자동 테스트 가능
* Remote 화면 Snapshot 테스트 가능

Remote 기능의 핵심 정의는 **“SSH 클라이언트를 Mdir4에 넣는다”가 아니라 “SSH/SFTP 파일 시스템을 Mdir4의 새로운 드라이브 유형으로 추가한다”**로 한다.
