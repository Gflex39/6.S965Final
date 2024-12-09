



## USER LEDS
set_property PACKAGE_PIN AR11 [ get_ports "led[0]" ]
set_property IOSTANDARD LVCMOS18 [ get_ports "led[0]" ]

set_property PACKAGE_PIN AW10 [ get_ports "led[1]" ]
set_property IOSTANDARD LVCMOS18 [ get_ports "led[1]" ]

set_property PACKAGE_PIN AT11 [ get_ports "led[2]" ]
set_property IOSTANDARD LVCMOS18 [ get_ports "led[2]" ]

set_property PACKAGE_PIN AU10 [ get_ports "led[3]" ]
set_property IOSTANDARD LVCMOS18 [ get_ports "led[3]" ]



## USER PUSH BUTTON
set_property PACKAGE_PIN AV12 [ get_ports "btn[0]" ]
set_property IOSTANDARD LVCMOS18 [ get_ports "btn[0]" ]

set_property PACKAGE_PIN AV10 [ get_ports "btn[1]" ]
set_property IOSTANDARD LVCMOS18 [ get_ports "btn[1]" ]

set_property PACKAGE_PIN AW9 [ get_ports "btn[2]" ]
set_property IOSTANDARD LVCMOS18 [ get_ports "btn[2]" ]

set_property PACKAGE_PIN AT12 [ get_ports "btn[3]" ]
set_property IOSTANDARD LVCMOS18 [ get_ports "btn[3]" ]





## PMOD


set_property -dict {PACKAGE_PIN AW15 IOSTANDARD LVCMOS18}  [ get_ports "gem_tx_clk" ]
set_property -dict {PACKAGE_PIN AK17 IOSTANDARD LVCMOS18}  [ get_ports "gem_tx[0]" ]
set_property -dict {PACKAGE_PIN AJ16 IOSTANDARD LVCMOS18}  [ get_ports "gem_tx[1]" ]
set_property -dict {PACKAGE_PIN AF17 IOSTANDARD LVCMOS18}  [ get_ports "gem_tx[2]" ]
set_property -dict {PACKAGE_PIN AG17 IOSTANDARD LVCMOS18}  [ get_ports "gem_tx[3]" ]
set_property -dict {PACKAGE_PIN AU14 IOSTANDARD LVCMOS18}  [ get_ports "gem_tx_ctrl" ]

set_property -dict {PACKAGE_PIN AF16 IOSTANDARD LVCMOS18}  [ get_ports "gem_mdio" ]
set_property -dict {PACKAGE_PIN AF15 IOSTANDARD LVCMOS18}  [ get_ports "gem_mdc" ]

set_property -dict {PACKAGE_PIN AW16 IOSTANDARD LVCMOS18}  [ get_ports "gem_rx[0]" ]
set_property -dict {PACKAGE_PIN AR13 IOSTANDARD LVCMOS18}  [ get_ports "gem_rx[1]" ]
set_property -dict {PACKAGE_PIN AT15 IOSTANDARD LVCMOS18}  [ get_ports "gem_rx[2]" ]
set_property -dict {PACKAGE_PIN AU13 IOSTANDARD LVCMOS18}  [ get_ports "gem_rx[3]" ]
set_property -dict {PACKAGE_PIN AV16 IOSTANDARD LVCMOS18}  [ get_ports "gem_rx_clk" ]
set_property -dict {PACKAGE_PIN AV13 IOSTANDARD LVCMOS18}  [ get_ports "gem_rx_ctrl" ]




set_property BITSTREAM.CONFIG.UNUSEDPIN PULLUP [current_design]
set_property BITSTREAM.CONFIG.OVERTEMPSHUTDOWN ENABLE [current_design]
set_property BITSTREAM.GENERAL.COMPRESS TRUE [current_design]




