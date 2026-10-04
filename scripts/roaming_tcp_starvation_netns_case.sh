# shellcheck shell=bash
# One blackholed outer carrier must not stop healthy inner UDP flows.
run_tcp_starvation_case() {
  local port
  [ "$MULTIPATH_MODE" = fixed ] || return 2
  ip netns exec "$SRV_NS" python3 "$SCRIPT_DIR/roaming_bonding_udp_probe.py" server --seconds 30 \
    >"$WORK/starvation-echo-server.log" 2>&1 &
  LOAD_JOB_PID=$!
  sleep 1
  ip netns exec "$CLI_NS" timeout 8 python3 "$SCRIPT_DIR/roaming_bonding_udp_probe.py" probe --seconds 3 \
    >"$WORK/starvation-baseline.log" || return 1
  check "sixteen private echo flows work before a carrier is stalled" \
    "python3 -c 'import json; v=json.load(open(\"$WORK/starvation-baseline.log\")); assert v[\"distinct_echo_flows\"]==16 and v[\"received\"]>=300'"
  port=$(sed -n 's/.*Stream #1 JOINed session.*from 10\.40\.1\.2:\([0-9][0-9]*\).*/\1/p' "$WORK/server.log" | tail -n1)
  case "$port" in ''|*[!0-9]*) return 2;; esac
  ip netns exec "$RTR_NS" iptables -I FORWARD -i qrm-ar -o qrm-sr -p tcp --sport "$port" --dport 4443 -j DROP || return 1
  check "only one outer carrier is blackholed" \
    "ip netns exec $RTR_NS iptables -C FORWARD -i qrm-ar -o qrm-sr -p tcp --sport $port --dport 4443 -j DROP"
  ip netns exec "$CLI_NS" timeout 15 python3 "$SCRIPT_DIR/roaming_bonding_udp_probe.py" flood-probe --seconds 10 \
    >"$WORK/starvation-impaired.log" || return 1
  check "healthy echo flows keep progressing after the blocked carrier fills its queue" \
    "python3 -c 'import json; v=json.load(open(\"$WORK/starvation-impaired.log\")); assert v[\"last_half_received\"]>=40'"
  ip netns exec "$RTR_NS" iptables -D FORWARD -i qrm-ar -o qrm-sr -p tcp --sport "$port" --dport 4443 -j DROP || return 1
  check "the tunnel recovers after the selected carrier is unblocked" \
    "ip netns exec $CLI_NS ping -c5 -W2 10.88.0.1"
  check "bounded flood did not replace the client process or TUN" \
    "ip netns pids $CLI_NS | grep -qx '$CLIENT_PID' && test \"\$(ip netns exec $CLI_NS cat /sys/class/net/qrm0/ifindex)\" = '$TUN_IFINDEX'"
  kill "$LOAD_JOB_PID" 2>/dev/null || true
  wait "$LOAD_JOB_PID" 2>/dev/null || true
  LOAD_JOB_PID=
}
