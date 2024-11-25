
// Handle Torrent Download Form Submission
document.getElementById("download-form").addEventListener("submit", async function (event) {
    event.preventDefault();

    const torrentFile = document.getElementById("torrent_file_path").files[0];
    const outputPath = document.getElementById("output_path").value;

    if (!torrentFile || !outputPath) {
        alert("Please select a torrent file and provide an output path.");
        return;
    }

    const torrentFilePath = torrentFile.webkitRelativePath || torrentFile.name; // Adjust based on your setup

    const jsonBody = {
        torrent_file_path: torrentFilePath, // File name sent as string
        output_path: outputPath
    };

    const progressBarContainer = document.getElementById("progress-container");
    const progressBar = document.getElementById("progress-bar");
    progressBarContainer.style.display = "block";
    progressBar.style.width = "0%";
    progressBar.textContent = "0%";

    // Fake progress increment
    let progress = 0;
    const interval = setInterval(() => {
        if (progress < 90) {
            progress += 10; // Simulate loading progress
            progressBar.style.width = progress + "%";
            progressBar.textContent = progress + "%";
        }
    }, 200);

    try {
        const response = await fetch("http://127.0.0.1:8001/download", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(jsonBody),
        });

        clearInterval(interval);

        if (response.ok) {
            progressBar.style.width = "100%";
            progressBar.textContent = "100%";
            alert("Torrent downloaded successfully!");
        } else {
            progressBar.style.width = "100%";
            progressBar.textContent = "Error";
            alert("Error downloading torrent: " + (await response.text()));
        }
    } catch (error) {
        clearInterval(interval);
        progressBar.style.width = "100%";
        progressBar.textContent = "Error";
        console.error("Error:", error);
        alert("An error occurred while downloading the torrent.");
    }

    // Hide progress bar after completion
    setTimeout(() => {
        progressBarContainer.style.display = "none";
    }, 2000);
});

// Handle Magnet Download Form Submission
document.getElementById("magnet-form").addEventListener("submit", async function (event) {
    event.preventDefault();

    const magnetLink = document.getElementById("magnet_link").value;
    const outputPath = document.getElementById("magnet_output_path").value;

    if (!magnetLink || !outputPath) {
        alert("Please provide a magnet link and select an output path.");
        return;
    }

    const jsonBody = {
        magnet_link: magnetLink,
        magnet_output_path: outputPath
    };

    const progressBarContainer = document.getElementById("magnet-progress-container");
    const progressBar = document.getElementById("magnet-progress-bar");
    progressBarContainer.style.display = "block";
    progressBar.style.width = "0%";
    progressBar.textContent = "0%";

    // Fake progress increment
    let progress = 0;
    const interval = setInterval(() => {
        if (progress < 90) {
            progress += 10; // Simulate loading progress
            progressBar.style.width = progress + "%";
            progressBar.textContent = progress + "%";
        }
    }, 200);

    try {
        const response = await fetch("http://127.0.0.1:8001/magnet_download", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(jsonBody),
        });

        clearInterval(interval);

        if (response.ok) {
            progressBar.style.width = "100%";
            progressBar.textContent = "100%";
            alert("Magnet download completed successfully!");
        } else {
            progressBar.style.width = "100%";
            progressBar.textContent = "Error";
            alert("Error downloading magnet link: " + (await response.text()));
        }
    } catch (error) {
        clearInterval(interval);
        progressBar.style.width = "100%";
        progressBar.textContent = "Error";
        console.error("Error:", error);
        alert("An error occurred while downloading the magnet link.");
    }

    // Hide progress bar after completion
    setTimeout(() => {
        progressBarContainer.style.display = "none";
    }, 2000);
});



