# herdr-obsidian

[English](README.md) | [**한국어**](README.ko.md)

지원 언어: 한국어

Obsidian 앱을 열지 않고 herdr pane 하나에서 [Obsidian](https://obsidian.md) 볼트를 보고 편집합니다. 노트는 일반 Markdown 파일이라 어떤 폴더든 쓸 수 있고, Google Drive 데스크톱 앱으로 동기화된 폴더도 됩니다. 편집 내용은 자동으로 저장됩니다.

## 대시보드 디자인

왼쪽은 편집기입니다. 오른쪽에는 볼트 목록과 폴더·노트 목록이 있습니다. 아래 상태줄에는 메시지, 입력줄, `Ctrl+H` 힌트가 표시됩니다.

```text
┌ plan.md ● ──────────────────────────┐┌ 볼트 목록 (↑↓ Enter 전환) ─────────┐
│# Plan                               ││● C:\work\obsidian\herdr_test       │
│                                     ││  C:\work\obsidian\test             │
│- [ ] README 쓰기                    │└────────────────────────────────────┘
│- [x] 자동 저장                      │┌ C:\work\obsidian\herdr_test ───────┐
│                                     ││▾ Projects                          │
│> 노트는 일반 Markdown 입니다        ││    plan.md                         │
│                                     ││▸ Archive                           │
│                                     ││  welcome.md                        │
└─────────────────────────────────────┘└────────────────────────────────────┘
saved: C:\work\obsidian\herdr_test\Projects\plan.md             Ctrl+H 단축키
```

볼트 목록은 오른쪽 위 패널입니다. `↑`/`↓`로 볼트를 고르고 `Enter`를 누르면 전환되며, 아래 폴더·노트 목록이 그 볼트로 바뀝니다. 현재 볼트에는 `●`가 붙습니다. 제목, 인용, 목록, 코드 블록에는 색을 입히고, 글자는 쓴 그대로 보여 줍니다.

이 플러그인은 Agents 사이드바 블록 대신 pane 전체를 쓰는 터미널 앱입니다.

`Ctrl+H`를 누르면 단축키 목록이 팝업으로 열립니다. `Esc` 또는 `Ctrl+H`로 닫습니다.

## 설치

아직 `herdr-plugin.toml`이 없어서 `herdr plugin install`은 되지 않습니다. 빌드한 뒤 herdr pane에서 실행 파일을 실행합니다.

```sh
git clone https://github.com/vrwolf1004/herdr-obsidian.git
cd herdr-obsidian
cargo build --release
target/release/herdr-obsidian "C:\work\obsidian\herdr_test"
```

실행 인자로 준 볼트는 볼트 목록에 추가되고 바로 열립니다. 인자가 없으면 목록에서 실제로 있는 첫 볼트를 엽니다.

## 볼트 목록

목록은 `config.toml`에 저장됩니다.

| OS | 경로 |
| --- | --- |
| Windows | `%APPDATA%\herdr-obsidian\config.toml` |
| Linux, macOS | `$XDG_CONFIG_HOME/herdr-obsidian/config.toml` 또는 `~/.config/herdr-obsidian/config.toml` |

```toml
vaults = ['C:\work\obsidian\herdr_test', 'C:\work\obsidian\test']
```

## 단축키

| 키 | 기능 |
| --- | --- |
| `Ctrl+O` | 폴더 열기. 볼트 기준 상대 경로 또는 절대 경로 (볼트 밖도 가능) |
| `Ctrl+N` | 목록 루트에 새 노트 만들기 (`.md` 자동) |
| `Ctrl+G` | 선택한 폴더 안에 새 폴더 만들기 |
| `Ctrl+T` | 선택한 노트 이름 변경 |
| `Ctrl+D` | 선택한 노트를 `.trash/`로 이동 (`y`로 확인) |
| `Ctrl+S` | 바로 저장 |
| `Ctrl+Q` | 저장 후 종료 |
| `Ctrl+H` | 단축키 목록 열기/닫기 |
| `Esc` | 포커스 전환 (볼트 → 목록 → 편집기), 입력줄·팝업 닫기 |
| `↑` `↓` `Enter` `→` `←` | 이동, 노트 열기, 폴더 펼치기·접기 |
| `r` | 목록 새로고침 |
| 마우스 | 클릭으로 선택, 커서 이동, 휠로 스크롤 |

## 저장

- 마지막 입력 0.8초 뒤에 저장합니다.
- 임시 파일에 쓴 뒤 이름을 바꾸는 방식이라, 노트가 쓰다 만 상태로 남지 않습니다.
- 저장에 실패하면 3초마다 다시 시도합니다. `Ctrl+Q`를 두 번 누르면 저장하지 않고 종료합니다.
- 노트를 연 뒤 디스크의 파일이 바뀌면(Obsidian, 동기화 프로그램) 자동 저장을 멈추고 경고를 표시합니다. `Ctrl+S`를 누르면 편집 내용으로 덮어씁니다.
- 삭제한 노트는 목록 루트 아래 `.trash/`로 옮겨집니다. 복구 화면은 없으므로 파일을 직접 옮겨야 합니다.

## 릴리스 노트

### 0.5.0

- 볼트 목록(오른쪽 위): 볼트는 `config.toml`에서 읽습니다. `↑`/`↓`로 고르고 `Enter`로 전환합니다.
- 폴더 열기(`Ctrl+O`): 상대 경로는 볼트 안, 절대 경로는 볼트 밖도 가능합니다.
- 파일 작업: 노트 생성(`Ctrl+N`), 폴더 생성(`Ctrl+G`), 이름 변경(`Ctrl+T`), `.trash/` 이동(`Ctrl+D`).
- 단축키 팝업(`Ctrl+H`)과 상태줄 오른쪽의 `Ctrl+H` 힌트.
- 안전한 저장: 임시 파일 후 이름 바꾸기, 실패 시 3초마다 재시도, `Ctrl+Q` 두 번이면 저장 없이 종료.
- 외부 변경 감지: 디스크 파일이 바뀌면 자동 저장을 멈추고, `Ctrl+S`로 덮어씁니다.
- `←`로 펼쳐진 폴더를 접거나, 상위 폴더로 이동합니다.

## 요구 사항

- Windows, Linux, macOS (개발과 테스트는 Windows에서 했습니다)
- herdr 0.9.0 이상
- 빌드용 Rust (stable)

## 문서

- [사용법 (한국어)](docs/usage.ko.md)
- [Usage guide (English)](docs/usage.md)
- [TODO](docs/todo.md)

## 개발

```sh
cargo test
cargo clippy --all-targets
cargo fmt --check
```

## 라이선스

MIT

## 프로젝트 응원하기

프로젝트가 도움이 되셨다면 커피 한 잔으로 개발을 응원해 주세요.

[![Ko-fi에서 응원하기](https://img.shields.io/badge/Ko--fi-%EC%9D%91%EC%9B%90%ED%95%98%EA%B8%B0-FF5E5B?style=for-the-badge&logo=buymeacoffee&logoColor=white)](https://ko-fi.com/dev_ohyt)
