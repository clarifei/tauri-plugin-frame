import { invoke } from "@tauri-apps/api/core";

function App() {
	const openNewWindow = async () => {
		try {
			await invoke("open_new_window");
		} catch (e) {
			console.error("Failed to open new window:", e);
		}
	};

	return (
		<div className="container">
			<h1>Frame Plugin Test</h1>

			<div className="row">
				<a href="https://tauri.app" target="_blank">
					<img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
				</a>
			</div>

			<p>Using <code>auto_titlebar: true</code></p>
			
			<button onClick={openNewWindow} style={{ marginTop: "20px", padding: "10px 20px" }}>
				Open New Window
			</button>
			
			<p style={{ fontSize: "12px", marginTop: "10px", color: "#888" }}>
				New window will also have custom titlebar
			</p>
		</div>
	);
}

export default App;
