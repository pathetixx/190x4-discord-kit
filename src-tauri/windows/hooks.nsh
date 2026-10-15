; 190x4 Discord Kit — хуки NSIS.
; После установки/обновления: если автозапуск сторожа был включён — поднимаем
; сторож заново (инсталлер гасит запущенные копии приложения перед заменой exe).
; При удалении: убираем автозапуск и гасим сторожа.

!macro NSIS_HOOK_POSTINSTALL
  ReadRegStr $0 HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "190x4DiscordKit"
  StrCmp $0 "" +2
  Exec '"$INSTDIR\${MAINBINARYNAME}.exe" --guard'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::Exec 'taskkill /F /IM "${MAINBINARYNAME}.exe"'
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "190x4DiscordKit"
!macroend
