# 사용법

[English](usage.md)

## 볼트 목록

볼트는 `config.toml`에서 읽습니다. 파일은 첫 실행 때 만들어집니다.

| OS | 경로 |
| --- | --- |
| Windows | `%APPDATA%\herdr-obsidian\config.toml` |
| Linux, macOS | `$XDG_CONFIG_HOME/herdr-obsidian/config.toml` 또는 `~/.config/herdr-obsidian/config.toml` |

```toml
vaults = ['C:\work\obsidian\herdr_test', 'C:\work\obsidian\test']
```

- 실행 인자로 준 볼트는 목록에 없으면 추가되고 바로 열립니다.
- 인자가 없으면 목록에서 실제로 있는 첫 볼트를 엽니다.
- 폴더가 없는 볼트는 `(없음)`으로 표시됩니다.
- 목록은 인자로 볼트를 추가할 때만 바뀝니다. 항목을 지우려면 파일을 직접 고칩니다.

## 단축키

앱에서 `Ctrl+H`를 누르면 이 목록을 팝업으로 볼 수 있습니다.

| 키 | 기능 |
| --- | --- |
| `Ctrl+O` | 폴더 열기. 볼트 기준 상대 경로 또는 절대 경로 (볼트 밖도 가능) |
| `Ctrl+N` | 목록 루트에 새 노트 만들기 (`.md` 자동) |
| `Ctrl+G` | 선택한 폴더 안에 새 폴더 만들기 |
| `Ctrl+T` | 선택한 노트 이름 변경 |
| `Ctrl+D` | 선택한 노트를 `.trash/`로 이동 (`y`로 확인) |
| `Ctrl+S` | 바로 저장 (디스크에서 바뀐 파일도 덮어씀) |
| `Ctrl+Q` | 저장 후 종료 |
| `Ctrl+H` | 단축키 목록 열기/닫기 |
| `Esc` | 포커스 전환 (볼트 → 목록 → 편집기), 입력줄·팝업 닫기 |
| `↑` `↓` | 볼트 목록 또는 폴더·노트 목록 이동 |
| `Enter` / `→` | 노트 열기, 폴더 펼치기, 선택한 볼트로 전환 |
| `←` | 펼쳐진 폴더 접기, 아니면 상위 폴더로 이동 |
| `r` | 목록 새로고침 |
| 마우스 | 클릭으로 선택·커서 이동, 휠로 스크롤 |

단축키 글자는 영문 키로 인식합니다. 한글 입력 상태에서는 글자 키가 동작하지 않을 수 있으니 영문으로 바꿔 쓰세요.

## 저장

- 마지막 입력 0.8초 뒤에 저장합니다.
- 임시 파일에 쓴 뒤 이름을 바꾸는 방식이라, 노트가 쓰다 만 상태로 남지 않습니다.
- 저장에 실패하면 상태줄에 오류를 표시하고 3초마다 다시 시도합니다.
- 저장 실패 상태에서 `Ctrl+Q`를 누르면 경고가 뜹니다. 한 번 더 누르면 저장하지 않고 종료합니다.
- 노트를 연 뒤 디스크의 파일이 바뀌면(Obsidian, 동기화 프로그램) 자동 저장을 멈추고 경고를 표시합니다. `Ctrl+S`를 누르면 편집 내용으로 덮어씁니다.
- 삭제한 노트는 목록 루트 아래 `.trash/`로 옮겨집니다. 이름이 겹치면 UTC 시각을 붙입니다(`이름.20261112-055806.md`). 복구 화면은 없으므로 파일을 직접 옮겨야 합니다.

## 언어

화면은 en, ko, ru, fr, es, de, ja를 지원합니다. 언어는 다음 순서로 정합니다.

1. `HERDR_OBSIDIAN_LANG` 환경 변수
2. `config.toml`의 `lang`
3. OS 언어
4. `en`

선택한 언어에 없는 문구는 영어로 표시합니다. ru, fr, es, de, ja 번역은 초안이며 원어민 검수를 받지 않았습니다. 자세한 내용은 [i18n.md](i18n.md)를 보세요.

## 요구 사항

- Windows, Linux, macOS (개발과 테스트는 Windows에서 했습니다)
- herdr 0.9.0 이상
- 빌드용 Rust (stable)

## 개발

```sh
cargo test
cargo clippy --all-targets
cargo fmt --check
```

## 빌드와 실행

```sh
cargo build --release
target/release/herdr-obsidian "C:\work\obsidian\herdr_test"
```

실행 파일은 herdr pane에서 실행합니다. 아직 `herdr-plugin.toml`이 없어서 `herdr plugin install`은 되지 않습니다.
