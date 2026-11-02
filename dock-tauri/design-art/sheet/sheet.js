/* Dock 美术规范页：放大曲线 + 目录高亮。无依赖。 */

(function () {
  "use strict";

  var NS = "http://www.w3.org/2000/svg";

  /* ---------- 放大传播曲线 ---------- */
  // 与 src/dock/magnify.ts 完全同构：广义高斯，距离越远衰减越快。
  var PEAK = 1.6;
  var FALLOFF = 0.45;
  var RADIUS = 2.2; // 图标格数

  function scaleAt(distanceInIcons) {
    if (distanceInIcons >= RADIUS) return 1;
    var normalized = distanceInIcons / RADIUS;
    return 1 + (PEAK - 1) * Math.exp(-Math.pow(normalized / FALLOFF, 4));
  }

  var chart = document.getElementById("magnify-chart");
  if (chart) drawChart(chart);

  function drawChart(svg) {
    var X0 = 46, X1 = 404, Y0 = 18, Y1 = 142;
    var D_MAX = 2.4, S_MIN = 1.0, S_MAX = 1.65;

    var x = function (d) { return X0 + (d / D_MAX) * (X1 - X0); };
    var y = function (s) { return Y1 - ((s - S_MIN) / (S_MAX - S_MIN)) * (Y1 - Y0); };

    function el(name, attrs) {
      var node = document.createElementNS(NS, name);
      for (var key in attrs) node.setAttribute(key, attrs[key]);
      return node;
    }

    function text(px, py, content, anchor, fill) {
      var node = el("text", {
        x: px,
        y: py,
        fill: fill || "#6d7690",
        "font-size": "9",
        "font-family": "Cascadia Mono, Consolas, monospace",
        "text-anchor": anchor || "start"
      });
      node.textContent = content;
      return node;
    }

    // 网格
    [1.0, 1.2, 1.4, 1.6].forEach(function (level) {
      svg.appendChild(
        el("line", {
          x1: X0, y1: y(level), x2: X1, y2: y(level),
          stroke: "rgba(255,255,255,0.06)", "stroke-width": "1"
        })
      );
      svg.appendChild(text(X0 - 10, y(level) + 3, level.toFixed(1), "end"));
    });

    [0, 0.5, 1, 1.5, 2].forEach(function (tick) {
      svg.appendChild(
        el("line", {
          x1: x(tick), y1: Y0, x2: x(tick), y2: Y1,
          stroke: "rgba(255,255,255,0.04)", "stroke-width": "1"
        })
      );
      svg.appendChild(text(x(tick), Y1 + 16, tick + "", "middle"));
    });
    svg.appendChild(text(X1, Y1 + 34, "distance, icons", "end"));
    svg.appendChild(text(X0 - 10, Y0 - 6, "scale", "end"));

    // 曲线
    var points = [];
    for (var i = 0; i <= 160; i++) {
      var d = (i / 160) * D_MAX;
      points.push([x(d), y(scaleAt(d))]);
    }

    var line = points
      .map(function (p, index) {
        return (index === 0 ? "M" : "L") + p[0].toFixed(2) + " " + p[1].toFixed(2);
      })
      .join("");

    var area =
      "M" + x(0).toFixed(2) + " " + Y1 +
      " L" + points.map(function (p) { return p[0].toFixed(2) + " " + p[1].toFixed(2); }).join(" L") +
      " L" + x(D_MAX).toFixed(2) + " " + Y1 + " Z";

    svg.appendChild(el("path", { d: area, fill: "rgba(124,155,224,0.10)" }));
    svg.appendChild(
      el("path", {
        d: line,
        fill: "none",
        stroke: "#7c9be0",
        "stroke-width": "2",
        "stroke-linecap": "round",
        "stroke-linejoin": "round"
      })
    );

    // 关键点：靠右的标签改为左对齐，避免被 viewBox 裁掉
    [
      { d: 0, label: "1.600" },
      { d: 1, label: "1.212" },
      { d: 2.2, label: "1.000" }
    ].forEach(function (mark) {
      var cx = x(mark.d);
      var cy = y(scaleAt(mark.d));
      var flip = cx > 350;
      svg.appendChild(
        el("circle", { cx: cx, cy: cy, r: "3.2", fill: "#7c9be0", stroke: "#0b0e15", "stroke-width": "1.6" })
      );
      svg.appendChild(
        text(flip ? cx - 8 : cx + 8, cy - 7, mark.label, flip ? "end" : "start", "#98a1b6")
      );
    });
  }

  /* ---------- 目录高亮 ---------- */
  var links = Array.prototype.slice.call(document.querySelectorAll(".rail-nav a"));
  var sections = links
    .map(function (link) { return document.querySelector(link.getAttribute("href")); })
    .filter(Boolean);

  if (!("IntersectionObserver" in window) || sections.length === 0) return;

  var map = new Map();
  sections.forEach(function (section, index) { map.set(section, links[index]); });

  var observer = new IntersectionObserver(
    function (entries) {
      entries.forEach(function (entry) {
        if (!entry.isIntersecting) return;
        links.forEach(function (link) { link.classList.remove("is-active"); });
        var link = map.get(entry.target);
        if (link) link.classList.add("is-active");
      });
    },
    { rootMargin: "-20% 0px -70% 0px", threshold: 0 }
  );

  sections.forEach(function (section) { observer.observe(section); });

  // 首屏（封面）没有对应小节，先点亮第一项，避免目录一直无选中。
  if (!links.some(function (link) { return link.classList.contains("is-active"); })) {
    links[0].classList.add("is-active");
  }
})();
