# 0162. Rust 버전을 숫자로 고정한다

- 날짜: 2026-11-13
- 상태: **결정**(사장 2026-11-13 "Rust 버전 고정하자")
- 바탕: KB `reference/github-actions/pin-actions-and-rust-toolchain.md`(nail에서 해 본 방법과 함정).

## 왜
2026-11-12, Rust 1.100.0이 나온 뒤 첫 push 둘의 개발판 배포가 clippy에서 깨졌다(`needless_bool` — 그 커밋들이 손대지 않은 `app/play.rs`). 로컬은 1.99.1이라 같은 검사를 통과한 뒤였다. `rust-toolchain.toml`의 `channel = "stable"`과 워크플로의 `dtolnay/rust-toolchain@stable`은 움직이는 이름이다 — 검사한 것과 CI가 돌리는 것이 다른 날이 생긴다.

## 정한 것
- `rust-toolchain.toml`: `channel = "1.100.0"`. 버전 숫자는 **이 한 줄에만** 있다.
- CI: 워크플로 넷(deploy-web · build-android · build-desktop · build-ios)은 `dtolnay/rust-toolchain`을 직접 부르지 않고 로컬 묶은 액션 `.github/actions/rust`를 부른다 — 그 액션이 toml의 `channel` 줄을 읽어 `toolchain:` 입력으로 넘긴다(대상·구성 요소는 워크플로가 전처럼 덧붙인다).
- toml의 `targets`에 웹(wasm32)과 커밋 전 clippy가 보는 셋(Android · Windows · iOS)을 적는다 — 새 컨테이너에서도 rustup이 저절로 받아 "커밋 전 검사"가 바로 돈다(확인함: 목록에 더하자 다음 cargo 호출이 받았다).
- 올리는 법: 그 줄만 바꾸고 → 로컬에서 커밋 전 검사를 다 돌리고(새 clippy 지적을 그 커밋에서 고친다) → 커밋. 올리는 때는 우리가 정한다.

## 지킬 것(KB의 함정)
- 액션이 까는 툴체인과 toml이 가리키는 툴체인은 **같은 이름**이어야 한다. 워크플로에 `dtolnay/rust-toolchain@stable`이 하나라도 남으면 그 워크플로에서만 `can't find crate for 'std'`(덧붙인 대상이 엉뚱한 툴체인에 깔린다). 새 워크플로를 만들면 `grep -rn 'dtolnay/rust-toolchain@' .github` — 묶은 액션 안의 한 줄만 나와야 한다.
- 묶은 액션은 checkout 뒤에만 쓸 수 있다.
- 이번에 하지 않은 것: 액션들의 SHA 고정·Dependabot(KB 같은 노트의 앞 절) — Rust 버전만 고정했다.
