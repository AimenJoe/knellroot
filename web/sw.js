// 서비스 워커(0062, nail 0269의 틀 — KB reference/web/pwa-install-fullscreen-orientation.md): 설치한 앱·두 번째 실행부터 오프라인에서도 뜨게.
//
// 캐시 이름에 빌드 SHA가 들어간다(tools/web/assemble.sh가 __BUILD__ 를 build.txt와 같은 값으로 바꾼다). knellroot.wasm에 해시가 없어
// 오래 두는 캐시는 금지 — 배포마다 sw.js 바이트가 바뀌어 새 워커가 깔리고, 새 캐시를 채운 뒤 옛 캐시를 지운다.
// - skipWaiting을 쓰지 않는다: 새 판은 다음 실행부터(실행 중인 게임에 새 파일이 섞이지 않게).
// - 페이지·build.txt·매니페스트는 네트워크 먼저(못 받으면 캐시), 나머지는 이 판 캐시 먼저(없으면 받아서 담는다).
// - 기록(localStorage)은 건드리지 않는다.
const BUILD = "__BUILD__";
const CACHE = "knell-" + BUILD;
const PRECACHE = ["./", "index.html", "mq_js_bundle.js", "macroquad-gamepads-0.1.js", "knellroot.wasm", "manifest.webmanifest",
  "icons/icon-192.png", "icons/icon-512.png", "icons/maskable-512.png", "icons/apple-touch-icon.png", "icons/bell-128.png"];
const NETWORK_FIRST = /(\/|index\.html|build\.txt|manifest\.webmanifest)$/;

self.addEventListener("install", (e) => {
  e.waitUntil(
    caches.open(CACHE).then((c) =>
      // 하나가 실패해도 설치는 끝낸다(못 받은 것은 처음 쓸 때 받는다).
      Promise.all(PRECACHE.map((u) => c.add(new Request(u, { cache: "reload" })).catch(() => {})))
    )
  );
});

self.addEventListener("activate", (e) => {
  e.waitUntil(
    caches.keys().then((keys) => Promise.all(keys.filter((k) => k.startsWith("knell-") && k !== CACHE).map((k) => caches.delete(k))))
  );
});

self.addEventListener("fetch", (e) => {
  const req = e.request;
  if (req.method !== "GET") return;
  const url = new URL(req.url);
  if (url.origin !== location.origin) return;
  if (req.mode === "navigate" || NETWORK_FIRST.test(url.pathname)) {
    e.respondWith(
      fetch(req)
        .then((res) => {
          if (res.ok) {
            const copy = res.clone();
            caches.open(CACHE).then((c) => c.put(req, copy));
          }
          return res;
        })
        .catch(() => caches.match(req, { ignoreSearch: true }).then((hit) => hit || caches.match("index.html")))
    );
    return;
  }
  e.respondWith(
    caches.open(CACHE).then((c) =>
      c.match(req, { ignoreSearch: true }).then(
        (hit) =>
          hit ||
          fetch(req).then((res) => {
            if (res.ok && res.status === 200) c.put(req, res.clone());
            return res;
          })
      )
    )
  );
});
