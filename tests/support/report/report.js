(function () {
  "use strict";
  var root = document.documentElement;
  var controls = document.getElementById("report-controls");
  var rows = Array.prototype.slice.call(document.querySelectorAll("tr[data-case]"));
  var frames = Array.prototype.slice.call(document.querySelectorAll(".scroll"));
  var search = document.getElementById("case-search");
  var groupFilter = document.getElementById("group-filter");
  var styleFilter = document.getElementById("style-filter");
  var differencesOnly = document.getElementById("differences-only");
  var count = document.querySelector("[data-visible-count]");
  var empty = document.getElementById("no-results");
  var total = rows.length;
  var openRow = null;
  var collapsedGroups = Object.create(null);
  var groups = Array.prototype.slice
    .call(document.querySelectorAll("[data-group-row]"))
    .map(function (header) {
      var value = header.getAttribute("data-group-row");
      return {
        value: value,
        header: header,
        button: header.querySelector("[data-group-toggle]"),
        count: header.querySelector("[data-group-count]"),
        total: 0,
        matched: 0,
      };
    });
  var groupsByValue = Object.create(null);
  groups.forEach(function (group) {
    groupsByValue[group.value] = group;
  });
  rows.forEach(function (row) {
    var group = groupsByValue[row.dataset.group];
    if (group) {
      group.total += 1;
    }
  });

  function groupExpanded(value) {
    return collapsedGroups[value] !== true;
  }

  function expandGroup(value) {
    delete collapsedGroups[value];
  }

  function detailRow(row) {
    return document.getElementById(row.getAttribute("data-detail"));
  }

  function collapse(row) {
    var detail = detailRow(row);
    var button = row.querySelector(".case-toggle");
    if (detail) {
      detail.hidden = true;
    }
    if (button) {
      button.setAttribute("aria-expanded", "false");
    }
    row.dataset.expanded = "false";
  }

  function toggle(row) {
    var detail = detailRow(row);
    var button = row.querySelector(".case-toggle");
    if (!detail || !button) {
      return;
    }
    if (openRow && openRow !== row) {
      collapse(openRow);
    }
    if (row.dataset.expanded === "true") {
      collapse(row);
      openRow = null;
      return;
    }
    detail.hidden = false;
    button.setAttribute("aria-expanded", "true");
    row.dataset.expanded = "true";
    openRow = row;
  }

  function matchesFilters(row) {
    if (groupFilter && groupFilter.value !== "all" && row.dataset.group !== groupFilter.value) {
      return false;
    }
    if (styleFilter.value !== "all" && row.dataset.style !== styleFilter.value) {
      return false;
    }
    if (differencesOnly.checked && row.dataset.status !== "fail") {
      return false;
    }
    var tokens = search.value.toLowerCase().split(/\s+/).filter(Boolean);
    return tokens.every(function (token) {
      return row.dataset.search.indexOf(token) !== -1;
    });
  }

  function applyFilters() {
    var visible = 0;
    groups.forEach(function (group) {
      group.matched = 0;
    });
    rows.forEach(function (row) {
      var group = groupsByValue[row.dataset.group];
      var matched = matchesFilters(row);
      if (matched && group) {
        group.matched += 1;
      }
      var ok = matched && groupExpanded(row.dataset.group);
      if (!ok && row === openRow) {
        collapse(row);
        openRow = null;
      }
      row.hidden = !ok;
      if (ok) {
        visible += 1;
      }
    });
    groups.forEach(function (group) {
      group.header.hidden = group.matched === 0;
      group.count.textContent =
        group.matched === group.total
          ? group.total + " cases"
          : group.matched + " / " + group.total + " cases";
      if (group.button) {
        group.button.setAttribute(
          "aria-expanded",
          groupExpanded(group.value) ? "true" : "false",
        );
      }
    });
    count.hidden = visible === total;
    count.textContent = visible + " / " + total + " cases";
    empty.hidden = groups.some(function (group) {
      return group.matched !== 0;
    });
  }

  function applyZoom() {
    var checked = document.querySelector('input[name="zoom"]:checked');
    var zoom = checked ? checked.value : "1";
    root.style.setProperty("--zoom", zoom);
    root.dataset.zoom = zoom;
    frames.forEach(function (frame) {
      if (frame.scrollWidth > frame.clientWidth + 1) {
        frame.setAttribute("tabindex", "0");
      } else {
        frame.removeAttribute("tabindex");
      }
    });
  }

  function groupFromUrl() {
    return new URLSearchParams(location.hash.slice(1)).get("group") ||
      new URLSearchParams(location.search).get("group");
  }

  function selectGroupFromUrl() {
    if (!groupFilter) {
      return;
    }
    var requested = groupFromUrl();
    if (requested === null) {
      return;
    }
    var options = Array.prototype.slice.call(groupFilter.options);
    var match = options.filter(function (option) {
      return option.value === requested;
    })[0];
    if (match) {
      groupFilter.value = requested;
      expandGroup(requested);
    }
  }

  function syncGroupToUrl() {
    if (!groupFilter) {
      return;
    }
    var hash = groupFilter.value === "all" ? "" : "#group=" + encodeURIComponent(groupFilter.value);
    try {
      history.replaceState(null, "", location.pathname + location.search + hash);
    } catch (error) {
      location.hash = hash;
    }
  }

  var infoButton = document.getElementById("info-button");
  var infoPanel = document.getElementById("report-info");
  if (infoButton && infoPanel) {
    function positionInfo() {
      var buttonRect = infoButton.getBoundingClientRect();
      var viewportWidth = document.documentElement.clientWidth;
      var panelWidth = Math.min(400, viewportWidth - 24);
      infoPanel.style.left = Math.max(12, Math.min(buttonRect.right - panelWidth, viewportWidth - panelWidth - 12)) + "px";
      infoPanel.style.right = "auto";
      infoPanel.style.top = Math.max(12, buttonRect.bottom + 8) + "px";
    }
    infoButton.addEventListener("click", positionInfo);
    window.addEventListener("resize", positionInfo);
    if ("popover" in HTMLElement.prototype) {
      infoPanel.addEventListener("toggle", function (event) {
        infoButton.setAttribute("aria-expanded", event.newState === "open" ? "true" : "false");
      });
    } else {
      var setInfoOpen = function (open) {
        infoPanel.dataset.open = open ? "true" : "false";
        infoButton.setAttribute("aria-expanded", open ? "true" : "false");
      };
      infoButton.addEventListener("click", function () {
        setInfoOpen(infoPanel.dataset.open !== "true");
      });
      infoPanel.addEventListener("click", function (event) {
        if (event.target.closest(".info-close")) {
          setInfoOpen(false);
        }
      });
      document.addEventListener("keydown", function (event) {
        if (event.key === "Escape" && infoPanel.dataset.open === "true") {
          setInfoOpen(false);
          infoButton.focus();
        }
      });
    }
  }

  rows.forEach(function (row) {
    var button = row.querySelector(".case-toggle");
    if (button) {
      button.addEventListener("click", function () {
        toggle(row);
      });
    }
  });
  controls.addEventListener("change", function () {
    applyZoom();
    applyFilters();
  });
  if (groupFilter) {
    groupFilter.addEventListener("change", function () {
      if (groupFilter.value !== "all") {
        expandGroup(groupFilter.value);
      }
      syncGroupToUrl();
    });
  }
  groups.forEach(function (group) {
    if (!group.button) {
      return;
    }
    group.button.addEventListener("click", function () {
      if (groupExpanded(group.value)) {
        collapsedGroups[group.value] = true;
        if (openRow && openRow.dataset.group === group.value) {
          collapse(openRow);
          openRow = null;
        }
      } else {
        expandGroup(group.value);
      }
      applyFilters();
    });
  });
  search.addEventListener("input", applyFilters);
  selectGroupFromUrl();
  applyZoom();
  applyFilters();
})();
