
// This is the code used in the ad frame loaded from the device itself.
// It ís responsible for injecting the ad specific behavior sent by the app.

document.addEventListener("DOMContentLoaded", () => {
    document.body.addEventListener("keydown", event => {
        // Manage the backspace key to notify the parent frame that it should
        // regain the focus.
        if (event.key == "Backspace" || event.key == "EndCall") {
            event.preventDefault();
            window.parent.postMessage("ad-frame-exit", "*");
        }
    }, false);

    // Notify the parent frame that we are ready to receive the profile data.
    window.parent.postMessage("ad-frame-ready", "*");
});

var load = (function () {
    // Function which returns a function: https://davidwalsh.name/javascript-functions
    function _load(tag) {
        return function (url) {
            // This promise will be used by Promise.all to determine success or failure
            return new Promise(function (resolve, reject) {
                var element = document.createElement(tag);
                var parent = document.head;
                var attr = "src";

                // Important success and error for the promise
                element.onload = function () {
                    resolve(url);
                };
                element.onerror = function () {
                    reject(url);
                };

                // Need to set different attributes depending on tag type
                switch (tag) {
                    case "script":
                        element.async = true;
                        break;
                    case "link":
                        element.type = "text/css";
                        element.rel = "stylesheet";
                        attr = "href";
                }

                // Inject into document to kick off loading
                element[attr] = url;
                parent.appendChild(element);
            });
        };
    }

    return {
        css: _load("link"),
        js: _load("script")
    }
})();

window.addEventListener("message", function load_content(event) {
    let { content } = event.data;
    // The main frame is sending us the content and code to inject.
    document.body.innerHTML = content;
    // Evaluate the injected scripts.
    try {
        let remote_scripts = [];
        let eval_scripts = [];
        let scripts = document.querySelectorAll("script");
        for (var i = 0; i < scripts.length; i++) {
            let src = scripts[i].getAttribute("src");
            if (src && src.startsWith("http")) {
                remote_scripts.push(load.js(src));
            } else if (src !== "wrapper.js") {
                eval_scripts.push(scripts[i]);
            }
        }
        // Wait for all remote scripts to be loaded and then eval() the other ones.
        // TODO: decide on error management.
        Promise.all(remote_scripts).then(() => {
            eval_scripts.forEach(script => { eval(script.textContent) });
        });
    } catch (e) {
        console.error(e);
    }

    // Make sure we won't load anything else.
    window.removeEventListener("message", load_content);
}, false);
