const PLACEHOLDER_IMAGES = {
    profile_picture: {
        male: "https://storage.cloud.google.com/chat_server_local_development/placeholders/display_images/male.jpg",
        female: "https://storage.cloud.google.com/chat_server_local_development/placeholders/display_images/female.jpg",
        other: "https://storage.cloud.google.com/chat_server_local_development/placeholders/display_images/other.png"
    }
};

function clear_list_of_search_results() {
    document.getElementById("list-of-search-results").innerHTML = "";
}

(function() {
    const dropdowns = Array.from(document.querySelectorAll('.dropdown'));
    const dropdown_triggers = dropdowns.map(dropdown => {
        return document.querySelector(`button[aria-controls=${dropdown.id}]`);
    });

    dropdown_triggers.forEach((trigger) => {
        if (trigger === null) {
            return;
        }

        trigger.addEventListener('click', () => {
            const controls = trigger.getAttribute('aria-controls');
            const target = document.getElementById(controls);

            if (target.getAttribute("data-active") === "true") {
                target.setAttribute("data-active", "false");
                trigger.setAttribute("aria-expanded", "false");
            } else {
                target.setAttribute("data-active", "true");
                trigger.setAttribute("aria-expanded", "true");
            }
        });

        document.addEventListener("click", (ev) => {
            const target = ev.target;

            if (target === trigger || trigger.contains(target)) {
                return;
            }

            const controls = trigger.getAttribute('aria-controls');
            const target_dropdown = document.getElementById(controls);

            if (target_dropdown.getAttribute("data-active") === "true") {
                target_dropdown.setAttribute("data-active", "false");
                trigger.setAttribute("aria-expanded", "false");
            }
        });
    });
})()
