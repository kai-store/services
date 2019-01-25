# KaiOS Ads SDK

This SDK provides packaged apps the ability to embed Ads from 3rd party providers with the following characteristics:
- The Ad scripts don't inherit the app permissions.
- The Ad scripts don't need to be tailored for KaiOS.
- The integration effort on the application side is simple.

## Using the SDK in an app.

The resources from the sdk are served from a local HTTP server on the device, at the base url `http://local-device.kaiostech.com:8081/sdk/ads`.

Each application page that needs to embed an add needs to:
- Use the `embed-apps` permission if it needs to focus the ad frame (see https://bugzilla.kaiostech.com/show_bug.cgi?id=36739).
- Include the `ads-sdk.js` script.
- Create an iframe that will host the Ads.
- Configure the SDK.

Here's an example, loading ads from the Google ad network:

The html document:
```html
<!DOCTYPE html>
<html>

<head>
    <meta name="viewport" content="width=device-width, user-scalable=no, initial-scale=1">
    <meta charset="utf-8">

    <title>My App</title>
    <script src="http://127.0.0.1:8081/sdk/ads/ads-sdk.js"></script>
    <script src="main.js"></script>
    <link rel="stylesheet" href="style.css">
</head>

<body>
    <h1>Some content</h1>

    <iframe id="ad-frame" src="http://local-device.kaiostech.com:8081/sdk/ads/ad-wrapper.html" class="items" tabindex="2"></iframe>
</body>

</html>
```

And the `main.js` script:

```js
document.addEventListener("DOMContentLoaded", () => {
    // Check that we have access to the contacts api.
    let node = document.getElementById("count");
    if (navigator.mozContacts) {
        let req = navigator.mozContacts.getCount();
        req.onsuccess = () => {
            node.textContent = `${req.result} contacts`;
        }
    } else {
        node.textContent = "No access to contacts";
    }

    // Set the focus and start listening to key events.
    resetFocus(0);
    document.body.addEventListener("keydown", handleKeydownEvent);
});

// Very basic navigation/focus handler...
var currentIndex = 0;

function resetFocus(index) {
    currentIndex = index;
    document.querySelectorAll(".items")[currentIndex].focus();
}

function nav(move) {
    let old = currentIndex;
    var items = document.querySelectorAll(".items");
    var next = currentIndex + move;
    if (next < 0) {
        next = items.length - 1;
    }
    if (next == items.length) {
        next = 0;
    }
    var targetElement = items[next];
    targetElement.focus();
    currentIndex = next;
}

function handleKeydownEvent(e) {
    switch (e.key) {
        case "ArrowUp":
        case "ArrowLeft":
            nav(-1);
            break;
        case "ArrowDown":
        case "ArrowRight":
            nav(1);
            break;
    }
}
// End of the navigation/focus handler.

KaiAdsSdk.onready = () => {
    // Google Ads.
    let content = `
    <script src="https://www.googletagservices.com/tag/js/gpt.js"></script>
    <script>
      googletag.cmd.push(function() {
          googletag.defineSlot("/30497360/native_v2/iu0/iu1/iu2", [300, 250], "div-gpt-ad-1516592715092-0")
                   .addService(googletag.pubads());
          googletag.pubads().enableSingleRequest();
          googletag.enableServices();  });
    </script>
    <div id="div-gpt-ad-1516592715092-0" style="height:250px; width:300px;" tabindex="1">
    <script>
      googletag.cmd.push(function() {
          googletag.display("div-gpt-ad-1516592715092-0");
      });
    </script>
    `;

    return { content, target_frame: "ad-frame" }
}

KaiAdsSdk.onexit = () => {
    // Transfert the focus back to the main frame.
    resetFocus(currentIndex - 1);
}

KaiAdsSdk.init();
```

## Configuring the SDK

The `KaiAdsSdk` object can be setup with 2 callbacks: `onready` and `onexit`.

### The onready callback

This callback takes no parameter, and needs to return a `{ content, target }` object. It is called when the ad wrapper is ready.
The `content` property is the full content that will be injected in the ad frame, as a string.
The `target` property is the ID of the frame element in which to inject the content.

### The onexit callback

This callback is called when the focus can be transfered back to the main application from the ad frame.


## Initializing the SDK

Simply call `KaiAdsSdk.init()` to setup the SDK once it is configured.
