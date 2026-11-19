# 0253 — iOS 홈 화면 앱을 가로로 켜면 아래가 빈다: 높이를 lvh로

- 날짜: 2026-11-19
- 상태: 정함·**실기 확인**(사장 2026-11-19 "가로모드 버그는 iOS PWA 설치된 앱에서, 가로로 앱 시작하면 아래쪽에 공간이 뜨는 거야" → 고친 빌드에서 "수정된 것 확인").
  고치기 전 녹화(1284×2778): 가로의 아래 141px = 47pt가 그려지지 않았고, 전체 화면에 고정한 로딩 덮개도 같은 자리에서 끊겼다 — 상자 높이가 짧았다는 증거.
- 앞선 것: 0187(돌린 뒤 캔버스 다시 재기)·0243(처음부터 가로일 때 다 그린 순간 다시 재기) — 그리기 크기를 **상자**에 맞추는 고침이라,
  상자 자체가 짧으면 소용이 없었다.

## 원인(2차 출처 — 현장 보고, WebKit 공식 문서 없음)
`apple-mobile-web-app-status-bar-style: black-translucent` + `viewport-fit=cover`의 홈 화면 앱에서 WebKit은 화면 맨 위(상태 막대 밑)부터
그리지만 뷰포트 높이(`100%`·`innerHeight`·`dvh`·`svh`)는 상태 막대만큼 뺀 값으로 준다 — 아래에 그만큼 띠가 남는다. iOS 버전마다 띠의
크기가 다르게 보고된다(47pt·62pt). `100lvh`만 화면 전체 높이를 준다는 보고가 여럿이다.

## 고침(두 겹)
1. CSS: iOS(`@supports (-webkit-touch-callout: none)`) 홈 화면 앱(`display-mode: standalone/fullscreen`)에서만 `html, body, canvas`의
   높이를 `100vh` → `100lvh`. 브라우저 탭·Android·데스크톱은 그대로(`100%`).
2. 스크립트: `navigator.standalone`(iOS 홈 화면 앱만 참)일 때 상자 높이를 화면의 짧은 변(가로)·긴 변(세로)과 견줘 모자라면 px로 늘린다 —
   켤 때·load·resize·회전 뒤와 처음 3초 동안 몇 번. 늘면 0187의 `ResizeObserver`가 그리기 크기를 따라 맞춘다.
- nail도 같은 꼴이라 같이 넣었다(nail 0297).

## 확인
- 헤드리스 Chromium은 iOS 홈 화면 앱을 흉내 내지 못한다 — 웹이 그대로 뜨고(PWA 시험 통과) 스크립트 문법만 봤다. **실기 확인이 판정**(체크리스트 95).
- 메타 태그는 바꾸지 않았다(바꾸면 홈 화면에 다시 추가해야 한다는 보고가 있다). 새 빌드는 앱을 닫았다 다시 열어야 들어온다(서비스 워커).

## 출처
- https://github.com/openchamber/openchamber/issues/2287
- https://github.com/mrexodia/toilet-pi/pull/15
- https://github.com/tpdbf5509/busssss/pull/23
