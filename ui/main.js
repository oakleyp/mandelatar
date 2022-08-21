(function mandelatarUi() {
    const imgUrl = 'https://mandelatar.com/i1/random';
    const $mainImageEl = document.getElementById("main-image");
    const $mainImageLinkEl = document.getElementById("main-image-link");
    const $clipboardButton = document.getElementById("clipboard-button");


    // Form options
    const $enableProfCheckbox = document.getElementById("enable-profile");

    function initForm() {
        const params = new Proxy(new URLSearchParams(window.location.search), {
            get: (searchParams, prop) => searchParams.get(prop),
        });

        if (params['enable-profile']) {
            $enableProfCheckbox.checked = true;
        }
    }

    function respUrlToShortLink(respUrl) {
        const url = new URL(respUrl);
        url.pathname = url.pathname.replace("/api/v1/img/", "/i1/i/")

        return url;
    }

    function createImgUrlWithParams() {
        const url = new URL(imgUrl);

        if ($enableProfCheckbox.checked) {
            url.searchParams.append("overlay", "profile");
        }

        return url;
    }

    function updateImage() {
        var xhr = new XMLHttpRequest("MSXML2.XMLHTTP.3.0");
        xhr.responseType = 'blob';
        xhr.open('GET', createImgUrlWithParams(), true);

        xhr.onreadystatechange = function () {
            if (this.readyState == 4 && this.status == 200) {
                const respUrl = xhr.responseURL;
                // $mainImageEl.setAttribute("src", respUrl);
                $mainImageLinkEl.setAttribute("value", respUrlToShortLink(respUrl));

                const bytes = xhr.response;

                var reader = new FileReader();

                reader.onload = (e) => {
                    $mainImageEl.src = e.target.result;
                }

                reader.readAsDataURL(new Blob([bytes]));
            }
        };

        xhr.send();
    }

    $clipboardButton.addEventListener("click", (e) => {
        $mainImageLinkEl.select();
        document.execCommand("copy");
    });

    initForm();
    updateImage();
})();