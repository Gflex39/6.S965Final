`timescale 1ns / 1ps
`default_nettype none // prevents system from inferring an undeclared logic (good practice)

//set_property -dict {PACKAGE_PIN F14 IOSTANDARD LVCMOS33}  [ get_ports "gem_mdio" ]
//set_property -dict {PACKAGE_PIN C11 IOSTANDARD LVCMOS33}  [ get_ports "gem_rx_clk" ]
//set_property -dict {PACKAGE_PIN J13 IOSTANDARD LVCMOS33}  [ get_ports "gem_mdc" ]
//set_property -dict {PACKAGE_PIN D11 IOSTANDARD LVCMOS33}  [ get_ports "gem_rx[0]" ]
//set_property -dict {PACKAGE_PIN G18 IOSTANDARD LVCMOS33}  [ get_ports "gem_rx[1]" ]
//set_property -dict {PACKAGE_PIN K16 IOSTANDARD LVCMOS33}  [ get_ports "gem_rx[2]" ]
//set_property -dict {PACKAGE_PIN K14 IOSTANDARD LVCMOS33}  [ get_ports "gem_rx[3]" ]
//set_property -dict {PACKAGE_PIN J15 IOSTANDARD LVCMOS33}  [ get_ports "gem_rx_ctrl" ]
module top_level(
  input wire clk_100mhz, //
  input wire [15:0] sw, //all 16 input slide switches
  input wire [3:0] btn, //all four momentary button switches
  output logic [15:0] led, //16 green output LEDs (located right above switches)
  output logic [2:0] rgb0, //rgb led
  output logic [2:0] rgb1, //rgb led
  input wire [3:0] gem_rx,
  input wire gem_rx_clk,
  input wire  gem_rx_ctrl,
  inout wire gem_mdio,
  output logic gem_mdc,
  output logic [3:0] gem_tx,
  output logic gem_tx_clk,
  output logic  gem_tx_ctrl,

  output logic [3:0] ss0_an,//anode control for upper four digits of seven-seg display
  output logic [3:0] ss1_an,//anode control for lower four digits of seven-seg display
  output logic [6:0] ss0_c, //cathode controls for the segments of upper four digits
  output logic [6:0] ss1_c
  );
  logic clk_count;
  logic clk_25mhz;
  logic [2400:0] temp;
  logic data_valid;


  logic [3:0] rx_sync1;
  logic [3:0] rx_sync2;
  logic clk_sync1;
  logic clk_sync2;

  logic ctrl_sync1; 
  logic ctrl_sync2;


  
  
  always_ff @(posedge clk_100mhz)begin
    clk_count <= clk_count + 1;
    if (clk_count) begin
      clk_25mhz <= ~clk_25mhz;
    end
    rx_sync1<=gem_rx;
    rx_sync2<=rx_sync1;
    clk_sync1<=gem_rx_clk;
    clk_sync2<=clk_sync1;
    ctrl_sync1<=gem_rx_ctrl;
    ctrl_sync2<=ctrl_sync1;
  end

  logic mdio; //; = 1;
  logic mdc; // = 1;

  assign gem_mdio = mdio;
  assign gem_mdc = mdc;

  //localparam MSG = 32'b0101_01111_00000_10_0010_0001_00_000000;
  logic [31:0] MSG = {16'b0101_01111_00000_10,sw};
  logic [31:0] read_msg = 32'b0101_01111_00000_zz_zzzz_zzzz_zzzz_zzzz;
  // local

  assign gem_tx_clk = clk_25mhz;
  assign gem_tx = sw[3:0];
  assign gem_tx_ctrl = btn[2];
  logic old;
  logic en;
  initial en = 1;
  initial mdc =0;
  logic [31:0] count;
  logic [3:0] f;
  initial f =0;
  initial count = 0;
  logic [31:0] output_val;
  logic [31:0] debug;
  logic old_valid;
  //assign led[3:0] = {gem_rx};
  //assign led[9:8] = {gem_mdio,gem_mdc};
  
  always_ff @(posedge clk_100mhz)begin
    old <= clk_sync2;
    old_valid <=ctrl_sync2;
    output_val[31:28]<=  f[3:0];
    
    if (btn[1])begin

      output_val<=0;
      count<=0;
      f<=0;

    end else begin

    
        
    
    // if(gem_mdc)begin
    if (clk_sync2 && ~old)begin
      
      if (ctrl_sync2)begin
        if(count < sw )begin
      //   // // if (gem_rx != ctrl)begin

          output_val[27:0]<= {debug[11:0],output_val[11:0],rx_sync2 };
      //   //   //led[11:8] <=led[15:12];
      //   //   //led[7:4] <=led[11:8];
      //   // // end
          count<=count+1;
        end
        end
        // if(debug[3:0]!=count[3:0])begin
        //   count<={count[28:0],debug[3:0]};
        // end
        if(data_valid)begin
          
          f<=f+1;
        //   // output_val[15:0]<= debug[15:0];
        end
        
      end  //else begin
        // output_val<=0;
      //   count<=0;
      // end
      // output_val<= debug;
    end
    // end
  end
  //shut up those rgb LEDs (active high):
  assign rgb1= 0;
  assign rgb0 = 0;
  /* have btnd control system reset */
  logic sys_rst;
  assign sys_rst = btn[0];
  logic old_btn3;

  always_ff @(posedge clk_100mhz)begin
    old_btn3 <= btn[3];
  end


  logic [6:0] ss_c;
  seven_segment_controller mssc(.clk_in( clk_100mhz),
                                  .rst_in(btn[0]),
                                  .val_in(output_val),
                                  .cat_out(ss_c),
                                  .an_out({ss0_an, ss1_an}));


    

    assign ss0_c = ss_c; //control upper four digit's cathodes!
    assign ss1_c = ss_c;



    
    ether_4 phy_to_mac (.clk(clk_100mhz),
                      .rx_clk(clk_sync2),
                      .rst(btn[0]),
                      .ether_rxd(rx_sync2),
                      .ether_crsdv(ctrl_sync2),
                      .m00_axis_tdata(temp),
                      .m00_axis_tvalid(data_valid),
                      .debug(debug));
                      



  spi_tx
       #(.DATA_WIDTH(32),
         .DATA_PERIOD(10)) uut
        ( .clk_in(clk_100mhz),
          .rst_in(sys_rst),
          .data_in((btn[2])? MSG: read_msg),
          .trigger_in(btn[3] & ~old_btn3),
          .data_out(mdio),
          .data_clk_out(mdc)
          // .read_out(output_val)
        );
endmodule // top_level



//don't worry about weird edge cases
//we want a clean 50% duty cycle clock signal
module spi_tx
       #(   parameter DATA_WIDTH = 8,
            parameter DATA_PERIOD = 100
        )
        ( input wire clk_in,
          input wire rst_in,
          input wire [DATA_WIDTH-1:0] data_in,
          input wire trigger_in,
          inout wire data_out,
          output logic data_clk_out,
          output logic sel_out,
          output logic [16:0] read_out
        );
logic [$clog2(DATA_WIDTH)-1:0] bit_counter;
    logic [$clog2(DATA_PERIOD)-1:0] bit_dur_counter;
    logic [DATA_WIDTH-1:0] data_buffer;
    logic [$clog2(DATA_PERIOD)-1:0] EVEN_PERIOD;
    logic state; // 0 if sending, same as sel_out
    initial read_out = 0;
    assign data_out = data_buffer[DATA_WIDTH-1-bit_counter];

    always_ff @(posedge clk_in)begin
        EVEN_PERIOD <= 2*(DATA_PERIOD/2);
        //sel_out <= state;
        if (rst_in)begin
            sel_out <= 1;
            //data_out <= 0;
            bit_counter <= 0;
            bit_dur_counter <= 0;
            data_clk_out = 1'b0;
            data_buffer <= 0;
        end else begin
            if (sel_out) begin //idle
                if (trigger_in) begin
                    sel_out <= 0;
                    //state <= 0;
                    data_clk_out = 1'b0;
                    bit_counter <= 0;
                    bit_dur_counter <= 0;
                    data_buffer <= data_in;
                end
            end else begin //transmitting
                if (bit_dur_counter == EVEN_PERIOD-1) begin
                    if (bit_counter == DATA_WIDTH-1) begin
                        sel_out <= 1;
                    end else begin
                        bit_dur_counter <= 0;
                        
                        bit_counter <= bit_counter + 1;

                    end
                    read_out<={read_out[15:1],data_buffer[DATA_WIDTH-1-bit_counter]};
                    data_clk_out <= 1'b0;
                end else begin
                    if (bit_dur_counter == (EVEN_PERIOD/2)-1) begin
                        data_clk_out <= 1'b1;
                    end
                    bit_dur_counter <= bit_dur_counter + 1;
                end
            end
        end
    end
endmodule



`default_nettype wire

