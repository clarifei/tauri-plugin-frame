(() => {
	const tauri = window.__TAURI__;
	if (!tauri) {
		console.warn("FRAME: Tauri API not found. Set withGlobalTauri: true in tauri.conf.json");
		return;
	}

	const win = tauri.window.getCurrentWindow();
	const invoke = tauri.core.invoke;

	const waitForTitlebar = (callback, maxAttempts = 50) => {
		let attempts = 0;
		const check = () => {
			const tbEl = document.querySelector("[data-tauri-frame-tb]");
			if (tbEl) {
				callback(tbEl);
			} else if (++attempts < maxAttempts) {
				requestAnimationFrame(check);
			}
		};
		check();
	};

	const createButton = (tbEl, id) => {
		const btn = document.createElement("button");
		btn.id = "frame-tb-" + id;
		Object.assign(btn.style, {
			width: "58px",
			height: "32px",
			border: "none",
			padding: "0",
			outline: "none",
			display: "flex",
			fontSize: "10px",
			fontWeight: "300",
			cursor: "default",
			boxShadow: "none",
			borderRadius: "0",
			alignItems: "center",
			justifyContent: "center",
			transition: "background 0.1s",
			backgroundColor: "transparent",
			textRendering: "optimizeLegibility",
			fontFamily: "'Segoe Fluent Icons', 'Segoe MDL2 Assets'"
		});

		let timer;
		const hoverBg = id === "close" ? "rgba(196,43,28,1)" : "rgba(0,0,0,0.2)";

		btn.onmouseenter = () => {
			btn.style.backgroundColor = hoverBg;
			if (id === "maximize") {
				timer = setTimeout(() => {
					win.setFocus().then(() => invoke("plugin:frame|show_snap_overlay"));
				}, 620);
			}
		};

		btn.onmouseleave = () => {
			btn.style.backgroundColor = "transparent";
			clearTimeout(timer);
		};

		if (id === "minimize") {
			btn.innerHTML = "\uE921";
			btn.ariaLabel = "Minimize window";
			btn.onclick = () => win.minimize();
		} else if (id === "maximize") {
			btn.innerHTML = "\uE922";
			btn.ariaLabel = "Maximize window";
			btn.onclick = () => { clearTimeout(timer); win.toggleMaximize(); };
			win.onResized(() => {
				win.isMaximized().then((max) => {
					btn.innerHTML = max ? "\uE923" : "\uE922";
					btn.ariaLabel = max ? "Restore window" : "Maximize window";
				});
			});
		} else if (id === "close") {
			btn.innerHTML = "\uE8BB";
			btn.ariaLabel = "Close window";
			btn.onclick = () => win.close();
		}

		tbEl.appendChild(btn);
	};

	waitForTitlebar((tbEl) => {
		if (tbEl.querySelector("[id^='frame-tb-']")) return;
		["minimize", "maximize", "close"].forEach((id) => createButton(tbEl, id));
	});
})();
