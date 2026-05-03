(function () {
    "use strict";

    var flowNodes = Array.prototype.slice.call(document.querySelectorAll(".pipeline-node"));

    flowNodes.forEach(function (node) {
        node.addEventListener("mouseenter", function () {
            flowNodes.forEach(function (item) {
                item.classList.remove("is-active");
            });
            node.classList.add("is-active");
        });
    });
})();
