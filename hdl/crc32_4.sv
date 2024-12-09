module crc32(
  input wire [3:0] axiid,
  input wire axiiv,
  output logic axiov,
  output logic [31:0] axiod,
  input wire rst,
  input wire clk,
  input wire rx_clk);



    assign axiov = 1;
    logic valid;
    logic old;

    assign valid = axiiv && ~old && rx_clk;

	


  reg [31:0] lfsr_q,lfsr_c;

  assign axiod = ~lfsr_q;

  always @(*) begin
    lfsr_c[0] = lfsr_q[28] ^ axiid[0];
    lfsr_c[1] = lfsr_q[28] ^ lfsr_q[29] ^ axiid[0] ^ axiid[1];
    lfsr_c[2] = lfsr_q[28] ^ lfsr_q[29] ^ lfsr_q[30] ^ axiid[0] ^ axiid[1] ^ axiid[2];
    lfsr_c[3] = lfsr_q[29] ^ lfsr_q[30] ^ lfsr_q[31] ^ axiid[1] ^ axiid[2] ^ axiid[3];
    lfsr_c[4] = lfsr_q[0] ^ lfsr_q[28] ^ lfsr_q[30] ^ lfsr_q[31] ^ axiid[0] ^ axiid[2] ^ axiid[3];
    lfsr_c[5] = lfsr_q[1] ^ lfsr_q[28] ^ lfsr_q[29] ^ lfsr_q[31] ^ axiid[0] ^ axiid[1] ^ axiid[3];
    lfsr_c[6] = lfsr_q[2] ^ lfsr_q[29] ^ lfsr_q[30] ^ axiid[1] ^ axiid[2];
    lfsr_c[7] = lfsr_q[3] ^ lfsr_q[28] ^ lfsr_q[30] ^ lfsr_q[31] ^ axiid[0] ^ axiid[2] ^ axiid[3];
    lfsr_c[8] = lfsr_q[4] ^ lfsr_q[28] ^ lfsr_q[29] ^ lfsr_q[31] ^ axiid[0] ^ axiid[1] ^ axiid[3];
    lfsr_c[9] = lfsr_q[5] ^ lfsr_q[29] ^ lfsr_q[30] ^ axiid[1] ^ axiid[2];
    lfsr_c[10] = lfsr_q[6] ^ lfsr_q[28] ^ lfsr_q[30] ^ lfsr_q[31] ^ axiid[0] ^ axiid[2] ^ axiid[3];
    lfsr_c[11] = lfsr_q[7] ^ lfsr_q[28] ^ lfsr_q[29] ^ lfsr_q[31] ^ axiid[0] ^ axiid[1] ^ axiid[3];
    lfsr_c[12] = lfsr_q[8] ^ lfsr_q[28] ^ lfsr_q[29] ^ lfsr_q[30] ^ axiid[0] ^ axiid[1] ^ axiid[2];
    lfsr_c[13] = lfsr_q[9] ^ lfsr_q[29] ^ lfsr_q[30] ^ lfsr_q[31] ^ axiid[1] ^ axiid[2] ^ axiid[3];
    lfsr_c[14] = lfsr_q[10] ^ lfsr_q[30] ^ lfsr_q[31] ^ axiid[2] ^ axiid[3];
    lfsr_c[15] = lfsr_q[11] ^ lfsr_q[31] ^ axiid[3];
    lfsr_c[16] = lfsr_q[12] ^ lfsr_q[28] ^ axiid[0];
    lfsr_c[17] = lfsr_q[13] ^ lfsr_q[29] ^ axiid[1];
    lfsr_c[18] = lfsr_q[14] ^ lfsr_q[30] ^ axiid[2];
    lfsr_c[19] = lfsr_q[15] ^ lfsr_q[31] ^ axiid[3];
    lfsr_c[20] = lfsr_q[16];
    lfsr_c[21] = lfsr_q[17];
    lfsr_c[22] = lfsr_q[18] ^ lfsr_q[28] ^ axiid[0];
    lfsr_c[23] = lfsr_q[19] ^ lfsr_q[28] ^ lfsr_q[29] ^ axiid[0] ^ axiid[1];
    lfsr_c[24] = lfsr_q[20] ^ lfsr_q[29] ^ lfsr_q[30] ^ axiid[1] ^ axiid[2];
    lfsr_c[25] = lfsr_q[21] ^ lfsr_q[30] ^ lfsr_q[31] ^ axiid[2] ^ axiid[3];
    lfsr_c[26] = lfsr_q[22] ^ lfsr_q[28] ^ lfsr_q[31] ^ axiid[0] ^ axiid[3];
    lfsr_c[27] = lfsr_q[23] ^ lfsr_q[29] ^ axiid[1];
    lfsr_c[28] = lfsr_q[24] ^ lfsr_q[30] ^ axiid[2];
    lfsr_c[29] = lfsr_q[25] ^ lfsr_q[31] ^ axiid[3];
    lfsr_c[30] = lfsr_q[26];
    lfsr_c[31] = lfsr_q[27];

  end // always

  always_ff @(posedge clk) begin
    old<=rx_clk;

    if(rst) begin
      lfsr_q <= {32{1'b1}};
    end
    else begin
      lfsr_q <= valid ? lfsr_c : lfsr_q;
    end
  end // always
endmodule // crc