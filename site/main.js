/* jir — site behaviour: copy buttons, the command playground, the JDK cards,
   and the footer version pill. No dependencies; the page reads fine without it. */
(function () {
  'use strict';

  function copyText(button, text) {
    if (!navigator.clipboard) return;
    navigator.clipboard.writeText(text).then(function () {
      var label = button.textContent;
      button.textContent = button.dataset.copied || 'Copied';
      setTimeout(function () { button.textContent = label; }, 1200);
    });
  }

  /* one-liner command blocks: <button data-copy="element-id"> */
  document.querySelectorAll('button[data-copy]').forEach(function (button) {
    button.addEventListener('click', function () {
      var target = document.getElementById(button.dataset.copy);
      if (target) copyText(button, target.textContent);
    });
  });

  /* command playground: pick a command chip, show its session */
  var chips = document.querySelectorAll('.cmdchip');
  var sessions = document.querySelectorAll('.pg-session');

  function showCommand(key) {
    chips.forEach(function (chip) { chip.classList.toggle('active', chip.dataset.command === key); });
    sessions.forEach(function (session) { session.classList.toggle('on', session.dataset.session === key); });
  }

  chips.forEach(function (chip) {
    chip.addEventListener('click', function () { showCommand(chip.dataset.command); });
  });

  /* feature tags and the "more versions" card jump to the playground */
  document.querySelectorAll('[data-goto]').forEach(function (trigger) {
    trigger.addEventListener('click', function () {
      showCommand(trigger.dataset.goto);
      var playground = document.getElementById('playground');
      if (playground) playground.scrollIntoView({ behavior: 'smooth', block: 'start' });
    });
  });

  /* command reference: click a row to copy its command */
  document.querySelectorAll('.tablewrap tbody tr').forEach(function (row) {
    row.addEventListener('click', function () {
      var command = row.querySelector('td code');
      if (!command || !navigator.clipboard) return;
      navigator.clipboard.writeText(command.textContent.trim()).then(function () {
        row.classList.add('done');
        setTimeout(function () { row.classList.remove('done'); }, 900);
      });
    });
  });

  /* version cards: picking one makes it the active JDK — the badge travels */
  var cards = document.querySelectorAll('.vercard:not(.more)');
  var badge = document.querySelector('.vercards .badge');

  function activate(card) {
    if (card.classList.contains('active')) return;
    var current = document.querySelector('.vercard.active');
    if (current) current.classList.remove('active');
    card.classList.add('active');
    if (badge) card.appendChild(badge);
  }

  cards.forEach(function (card) {
    card.addEventListener('click', function () { activate(card); });
    card.addEventListener('keydown', function (event) {
      if (event.key !== 'Enter' && event.key !== ' ') return;
      event.preventDefault();
      activate(card);
    });
  });

  /* the JDK cards and the pipeline rise in as they scroll into view */
  var groups = document.querySelectorAll('.vercards, .flow');
  var noMotion = window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  if (groups.length && window.IntersectionObserver && Element.prototype.animate && !noMotion) {
    var observer = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (!entry.isIntersecting) return;
        observer.unobserve(entry.target);
        Array.prototype.forEach.call(entry.target.children, function (item, index) {
          item.animate(
            [{ opacity: 0, transform: 'translateY(14px)' }, { opacity: 1, transform: 'none' }],
            { duration: 480, delay: index * 90, easing: 'ease-out', fill: 'backwards' }
          );
        });
      });
    }, { threshold: .15 });
    groups.forEach(function (group) { observer.observe(group); });
  }

  var pgCopy = document.getElementById('pg-copy');
  if (pgCopy) {
    pgCopy.addEventListener('click', function () {
      var cmd = document.querySelector('.pg-session.on .pg-cmd');
      if (cmd) copyText(pgCopy, cmd.textContent);
    });
  }

  /* footer version pill: fill in the latest release tag; "Releases" stays as
     the fallback when the API is unreachable or JavaScript is off. */
  var version = document.getElementById('version');
  if (version && window.fetch) {
    fetch('https://api.github.com/repos/rururunu/Jir/releases/latest')
      .then(function (response) { return response.ok ? response.json() : null; })
      .then(function (data) {
        if (data && data.tag_name) version.textContent = data.tag_name;
      })
      .catch(function () {});
  }
})();
