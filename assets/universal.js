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
