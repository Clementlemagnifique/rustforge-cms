document.querySelectorAll('[data-reader-theme]').forEach(button => {
  button.addEventListener('click', () => {
    document.body.classList.remove('dark-mode', 'sepia-mode');
    const theme = button.dataset.readerTheme;
    if (theme === 'dark' || theme === 'sepia') document.body.classList.add(`${theme}-mode`);
  });
});
document.querySelector('[data-reader-font]')?.addEventListener('click', () => {
  document.body.classList.toggle('sans-mode');
});
