(() => {
	if (document.querySelector("[data-tauri-frame-tb]")) return;

	const tb = document.createElement("div");
	tb.setAttribute("data-tauri-frame-tb", "");
	tb.setAttribute("role", "group");
	tb.setAttribute("aria-label", "Window controls");
	Object.assign(tb.style, {
		top: "0",
		left: "0",
		zIndex: "100",
		width: "100%",
		height: "32px",
		display: "flex",
		position: "fixed",
		alignItems: "end",
		justifyContent: "end",
		backgroundColor: "transparent"
	});

	const drag = document.createElement("div");
	Object.assign(drag.style, { width: "100%", height: "100%", background: "transparent" });
	drag.setAttribute("data-tauri-drag-region", "");
	tb.appendChild(drag);

	document.body.prepend(tb);
})();
