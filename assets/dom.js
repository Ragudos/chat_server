
/**
 * @param {HTMLInputElement} input 
 * @param {HTMLElement} container 
 */
function attach_uploaded_image_to_container(
    input,
    container,
) {
    /**
     * @type {AbortController | undefined}
     */
    let remove_btn_abort_controller;
    /**
     * @type {AbortController | undefined}
     */
    let file_reader_abort_controller;

    setInterval(() => {
        console.log(input.files);
    }, 1000);
    
    input.addEventListener("change", (e) => {
        remove_btn_abort_controller?.abort();
        file_reader_abort_controller?.abort();
        container.innerHTML = "";

        if (e.target.files.length === 0) {
            return;
        }

        const target = e.target;

        if (target instanceof HTMLInputElement) {
            const file_reader = new FileReader();

            file_reader_abort_controller = new AbortController();
            file_reader.readAsDataURL(e.target.files[0]);
            file_reader.addEventListener("load", () => {
                image.src = file_reader.result;
            }, { once: true, signal: file_reader_abort_controller.signal});

            const image = document.createElement("img");

            image.alt = "Uploaded image";
            image.classList.add("profile_picture_sm");

            const name = document.createElement("small");

            name.style.fontSize = "0.75rem";
            name.textContent = e.target.files[0].name;

            const remove_btn = document.createElement("button");

            remove_btn.classList.add("destructive", "icon");
            remove_btn.innerHTML = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="currentColor" class="w-4 h-4"><path fill-rule="evenodd" d="M5 3.25V4H2.75a.75.75 0 0 0 0 1.5h.3l.815 8.15A1.5 1.5 0 0 0 5.357 15h5.285a1.5 1.5 0 0 0 1.493-1.35l.815-8.15h.3a.75.75 0 0 0 0-1.5H11v-.75A2.25 2.25 0 0 0 8.75 1h-1.5A2.25 2.25 0 0 0 5 3.25Zm2.25-.75a.75.75 0 0 0-.75.75V4h3v-.75a.75.75 0 0 0-.75-.75h-1.5ZM6.05 6a.75.75 0 0 1 .787.713l.275 5.5a.75.75 0 0 1-1.498.075l-.275-5.5A.75.75 0 0 1 6.05 6Zm3.9 0a.75.75 0 0 1 .712.787l-.275 5.5a.75.75 0 0 1-1.498-.075l.275-5.5a.75.75 0 0 1 .786-.711Z" clip-rule="evenodd" /></svg>`;

            remove_btn_abort_controller = new AbortController();

            remove_btn.addEventListener("click", () => {
                remove_btn_abort_controller?.abort();
                container.innerHTML = "";
                target.value = "";
            }, {
                signal: remove_btn_abort_controller.signal,
                once: true
            });

            container.appendChild(image);
            container.appendChild(name);
            container.appendChild(remove_btn);
        }
    });
}
