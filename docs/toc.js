// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="introduction.html">Introduction</a></li><li class="chapter-item expanded affix "><a href="key-changes.html">Key Changes</a></li><li class="chapter-item expanded "><a href="guides.html">Guides</a><a class="toggle"><div>❱</div></a></li><li><ol class="section"><li class="chapter-item "><a href="guides/swap_tokens.html">Swap Tokens</a></li><li class="chapter-item "><div>Add/Remove Liquidity</div></li><li class="chapter-item "><div>Tracking Volume</div></li><li class="chapter-item "><div>Tracking Pool Balances</div></li><li class="chapter-item "><div>Finding the Best Quote</div></li><li class="chapter-item "><div>Bytes32 Decoding</div></li><li class="chapter-item "><a href="guides/price_from_id.html">Price From Bin Id</a></li><li class="chapter-item "><div>Bin Id From Price</div></li><li class="chapter-item "><div>Finding Liquidity Depth</div></li><li class="chapter-item "><div>User Balances</div></li></ol></li><li class="chapter-item expanded "><a href="concepts.html">Concepts</a><a class="toggle"><div>❱</div></a></li><li><ol class="section"><li class="chapter-item "><a href="concepts/concentrated_liquidity.html">Concentrated Liquidity</a></li><li class="chapter-item "><div>Bin Math</div></li><li class="chapter-item "><div>Bin Liquidity</div></li><li class="chapter-item "><div>Swaps</div></li><li class="chapter-item "><div>Fees</div></li><li class="chapter-item "><div>Oracle</div></li></ol></li><li class="chapter-item expanded "><a href="contracts.html">Contracts</a></li><li class="chapter-item expanded "><a href="deployments.html">Deployments</a><a class="toggle"><div>❱</div></a></li><li><ol class="section"><li class="chapter-item "><div>Secret Mainnet</div></li><li class="chapter-item "><a href="deployments/pulsar.html">Pulsar Testnet</a></li></ol></li><li class="chapter-item expanded "><div>SDK</div></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString();
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
