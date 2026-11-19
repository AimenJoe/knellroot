// 옛 주소의 서비스 워커 자리(0246). 게임을 옮겼으니 옛 워커를 걷는다: 바로 바꿔 들고 캐시를 지운 뒤 스스로 내려간다.
// 옛 워커는 페이지 이동을 네트워크 먼저로 받으므로(web/sw.js), 온라인이면 넘김 페이지가 먼저 뜬다 — 이것은 뒷정리다.
self.addEventListener("install", () => self.skipWaiting());
self.addEventListener("activate", (e) => {
  e.waitUntil(
    caches.keys()
      .then((ks) => Promise.all(ks.map((k) => caches.delete(k))))
      .then(() => self.registration.unregister())
  );
});
