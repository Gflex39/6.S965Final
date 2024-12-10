module idct_2d_dma #(
    parameter integer C_S00_AXIS_TDATA_WIDTH = 64,
    parameter integer C_M00_AXIS_TDATA_WIDTH = 64
) (
    // Ports of Axi Slave Bus Interface S00_AXIS
    input wire s00_axis_aclk,
    input wire s00_axis_aresetn,
    input wire s00_axis_tlast,
    input wire s00_axis_tvalid,
    input wire [C_S00_AXIS_TDATA_WIDTH-1 : 0] s00_axis_tdata,
    input wire [(C_S00_AXIS_TDATA_WIDTH/8)-1:0] s00_axis_tstrb,
    output logic s00_axis_tready,

    // Ports of Axi Master Bus Interface M00_AXIS
    input wire m00_axis_aclk,
    input wire m00_axis_aresetn,
    input wire m00_axis_tready,
    output logic m00_axis_tvalid,
    output logic m00_axis_tlast,
    output logic [C_M00_AXIS_TDATA_WIDTH-1 : 0] m00_axis_tdata,
    output logic [(C_M00_AXIS_TDATA_WIDTH/8)-1:0] m00_axis_tstrb
);
  localparam integer TOTAL_DATA_COUNT = 44160;  // = 8 * (3680 + 3680 / 4 + 3680 / 4)

  logic [16:0] data_ctr;  // 44160= (640 / 8) * (368 / 8) * 4 * 3. Total number of blocks * 4 rows / block * 3 channels

  // logic m00_axis_tvalid_reg;

  assign m00_axis_tvalid = s00_axis_tvalid;

  assign s00_axis_tready = m00_axis_tready || ~m00_axis_tvalid;
  assign m00_axis_tdata  = s00_axis_tdata;

  always_ff @(posedge s00_axis_aclk) begin : DMA_PROCESS
    if (~s00_axis_aresetn) begin
      data_ctr       <= 0;

      m00_axis_tlast <= 0;
      m00_axis_tstrb <= 8'b1111_1111;

      // m00_axis_tvalid_reg <= 0;
    end else begin
      // TODO: Implement DMA
      if (m00_axis_tready && s00_axis_tvalid) begin
        data_ctr <= data_ctr + 1;

        // m00_axis_tvalid_reg <= s00_axis_tvalid;

        if (data_ctr == TOTAL_DATA_COUNT - 1) begin
          m00_axis_tlast <= 1;
          data_ctr       <= 0;
        end else begin
          m00_axis_tlast <= 0;
        end
      end

      // m00_axis_tvalid_reg <= s00_axis_tvalid;
    end
  end

endmodule
