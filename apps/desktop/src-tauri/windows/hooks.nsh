!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Restoring Claude Desktop configuration..."
  IfFileExists "$INSTDIR\claude-plus-desktop.exe" 0 claude_plus_cleanup_done
  ExecWait '"$INSTDIR\claude-plus-desktop.exe" --claude-plus-restore-official' $0
  ${If} $0 == 0
    DetailPrint "Claude Desktop configuration restored."
  ${Else}
    DetailPrint "Claude Desktop configuration restore returned exit code $0."
  ${EndIf}
  claude_plus_cleanup_done:
!macroend
