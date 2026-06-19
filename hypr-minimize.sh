#!/bin/sh
_get_ws_status () {
  _ws_status=$(hyprctl workspaces -j | jq -r '.[] | select(.name == "special:minimized" )| .name');
}
_minimize_window () {
  hyprctl dispatch hl.dsp.window.move\('{ workspace = "special:minimized", follow = false }'\);
}
_restore_window () {
  _get_ws_status;
  if [ $(pgrep wofi) ]
      then kill -TERM $(pgrep wofi)
      exit 0
  fi
  if [ "$_ws_status" = "special:minimized" ]
  then win_addr=$(hyprctl clients -j | jq -r '.[] | select(.workspace.name == "special:minimized") | "\(.address) \(.title)@\(.class)"'|wofi --show dmenu -i -M fuzzy |cut -f1 -d\ |head -1);
  c_workspace_id=$(hyprctl activeworkspace -j | jq '.id');
  if echo "$win_addr" | grep -n ^0x >/dev/null
  then
  hyprctl dispatch hl.dsp.window.move\(\{workspace = "$c_workspace_id", follow = true, window = \"address:"$win_addr"\"\}\)
  fi
  else echo "Minimized window not found" | wofi --show dmenu -i -M fuzzy;
  fi
}
_minispace_stat () {
  _get_ws_status;
  if [ "$_ws_status" = "special:minimized" ]
  then _win_count=$(hyprctl workspaces -j | jq -r '.[] | select(.name == "special:minimized") | (.windows)');
  echo ":$_win_count";
  else echo "";
  fi
}
  case $1 in
    -m*) _minimize_window;;
    -r*) _restore_window;;
    -g*) _minispace_stat;;
    *)printf "Usage: hypr-minimize.sh <option> \n-m minimize an activite window \n-r open wofi menu for recovering minimized window \n-g get the status of minimize workspace\n";;
  esac
