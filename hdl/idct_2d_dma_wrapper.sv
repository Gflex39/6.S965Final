module idct_2d_dma #(
    parameter integer C_S00_AXIS_TDATA_WIDTH = 64,
    parameter integer C_M00_AXIS_TDATA_WIDTH = 64
) (
    // Ports of Axi Slave Bus Interface S00_AXIS
    input wire clk_in,
    input wire rst_in,
    // input wire s00_axis_tlast,
    input wire valid_in,
    input wire [C_S00_AXIS_TDATA_WIDTH-1 : 0] row_data_in,
    // input wire [(C_S00_AXIS_TDATA_WIDTH/8)-1:0] s00_axis_tstrb,
    // output logic s00_axis_tready,

    // Ports of Axi Master Bus Interface M00_AXIS
    // input wire m00_axis_aclk,
    // input wire m00_axis_aresetn,
    // input wire m00_axis_tready,
    // output logic m00_axis_tvalid,
    // output logic m00_axis_tlast,
    // output logic [C_M00_AXIS_TDATA_WIDTH-1 : 0] m00_axis_tdata,
    // output logic [(C_M00_AXIS_TDATA_WIDTH/8)-1:0] m00_axis_tstrb
);

  idct_2d_dma idctdma (
    .s00_axis_aclk(clk_in),
    .s00_axis_aresetn(rst_in),
    .s00_axis_tlast(),
    .s00_axis_tvalid(valid_in),
    .s00_axis_tdata(row_data_in),
    .s00_axis_tstrb(),
    .s00_axis_tready(),

    .m00_axis_aclk(clk_in),
    .m00_axis_areset(rst_in),
    .m00_axis_tready(),
    .m00_axis_tvali(),
    .m00_axis_tlast(),
    .[C_M00_AXIS_TDATA_WIDTH-1 : 0] m00_axis_tdata(),
    .[(C_M00_AXIS_TDATA_WIDTH/8)-1:0] m00_axis_tstrb()
  )

endmodule
