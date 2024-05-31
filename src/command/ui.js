const refreshCheckbox = Object.assign(document.createElement('input'), {
  type: 'checkbox',
  id: 'refresh-checkbox',
  checked: localStorage.getItem('auto-refresh') === '1',
})

refreshCheckbox.addEventListener('change', () => {
  localStorage.setItem('auto-refresh', refreshCheckbox.checked ? '1' : '0')
})

const label = Object.assign(document.createElement('label'), {for: refreshCheckbox.id, textContent: 'Auto'})

label.prepend(refreshCheckbox)


let blurTime = Date.now()

window.addEventListener('blur', () => {
  blurTime = Date.now()
})

window.addEventListener('focus', () => {
  if (refreshCheckbox.checked && Date.now() - blurTime > 5000) {
    window.location.reload()
  }
})


window.addEventListener('DOMContentLoaded', () => {
  document.querySelector('h1').parentNode.appendChild(label)
}, {once: true})
