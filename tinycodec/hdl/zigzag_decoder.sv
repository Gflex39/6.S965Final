module zigzag_decoder(
    input wire clk_in,
    input wire rst_in,
    input wire signed [11:0] value_in,
    input wire [5:0] run_in,
    input wire valid_in,

    output logic [95:0] column_out,
    output logic valid_out
);
    parameter WRITEA = 0;
    parameter READB = 0;
    parameter WRITEB = 1;
    parameter READA = 1;

    logic        state;
    logic        state1;
    logic        state2;
    logic        swap_buffers;

    logic [6:0]  wr_pos;
    logic [5:0]  wr_addr;
    logic        wr_valid;
    logic        wr_valid_late;
    logic [11:0] wr_value;
    logic [5:0]  wr_run;
    logic [5:0]  wr_mask_addr;

    logic        rd_enable;
    logic [3:0]  rd_left;
    logic [3:0]  rd_pos;
    logic        rd_valid;
    logic [2:0]  rd_pos3;

    logic [3:0]  addra;
    logic [7:0]  wea;
    logic [95:0] dina;

    logic [3:0]  addrb;
    logic [95:0] doutb;

    pipeline#(.PIPELINE_STAGES(1),.PIPELINE_WIDTH(19)) p1 (.clk_in(clk_in),.rst_in(rst_in),.signal_in({valid_in,run_in,value_in}),.signal_out({wr_valid,wr_run,wr_value}));
    pipeline#(.PIPELINE_STAGES(2),.PIPELINE_WIDTH(1))  p2 (.clk_in(clk_in),.rst_in(rst_in),.signal_in(valid_in),.signal_out(wr_valid_late));
    pipeline#(.PIPELINE_STAGES(1),.PIPELINE_WIDTH(1))  p3 (.clk_in(clk_in),.rst_in(rst_in),.signal_in(state),.signal_out(state1));
    pipeline#(.PIPELINE_STAGES(1),.PIPELINE_WIDTH(7))  p4 (.clk_in(clk_in),.rst_in(rst_in),.signal_in({state1,wr_addr}),.signal_out({state2,wr_mask_addr}));
    pipeline#(.PIPELINE_STAGES(2),.PIPELINE_WIDTH(1))  p5 (.clk_in(clk_in),.rst_in(rst_in),.signal_in(swap_buffers),.signal_out(rd_enable));
    pipeline#(.PIPELINE_STAGES(3),.PIPELINE_WIDTH(4))  p6 (.clk_in(clk_in),.rst_in(rst_in),.signal_in({(rd_left>0),rd_pos[2:0]}),.signal_out({rd_valid,rd_pos3}));

    scan_order_lut msol ( .clk_in(clk_in), .rst_in(rst_in), .x_in(wr_pos[5:0]+run_in), .x_out(wr_addr) );

    logic [63:0] mask_a;
    logic [63:0] mask_b;

    xilinx_true_dual_port_read_first_byte_write_2_clock_ram #(
        .NB_COL(8),
        .COL_WIDTH(12),
        .RAM_DEPTH(16)
    ) mbz (
        .addra(addra),
        .addrb(addrb),
        .dina(dina),
        .dinb(96'b0),
        .clka(clk_in),
        .clkb(clk_in),
        .wea(wea),
        .web(8'b0),
        .ena(1'b1),
        .enb(1'b1),
        .rsta(rst_in),
        .rstb(rst_in),
        .regcea(1'b1),
        .regceb(1'b1),
        .douta(),
        .doutb(doutb)
    );

    assign swap_buffers = valid_in && ((wr_pos + run_in + 1) >= 64);
    assign rd_pos = 8 - rd_left[2:0];

    always_ff @(posedge clk_in) begin
        if (rst_in) begin
            addra <= 0;
            addrb <= 0;
            mask_a <= 0;
            mask_b <= 0;
            wea <= 0;
            dina <= 0;
            state <= WRITEA;
            wr_pos <= 0;
            rd_left <= 0;
            column_out <= 0;
            valid_out <= 0;
        end else begin
            if (valid_in) begin // cycle0
                state <= swap_buffers ? ~state : state;
                wr_pos <= swap_buffers ? 0 : wr_pos + run_in + 1;

                if (swap_buffers) begin
                    case (state)
                        WRITEA: mask_b <= 0;
                        WRITEB: mask_a <= 0;
                    endcase
                end
            end

            if (wr_valid) begin // cycle1
                addra <= {state1, wr_addr[5:3]};

                case (wr_addr[2:0])
                    3'b000: begin wea <= 8'b00000001; dina <= {84'b0, wr_value       }; end
                    3'b001: begin wea <= 8'b00000010; dina <= {72'b0, wr_value, 12'b0}; end
                    3'b010: begin wea <= 8'b00000100; dina <= {60'b0, wr_value, 24'b0}; end
                    3'b011: begin wea <= 8'b00001000; dina <= {48'b0, wr_value, 36'b0}; end
                    3'b100: begin wea <= 8'b00010000; dina <= {36'b0, wr_value, 48'b0}; end
                    3'b101: begin wea <= 8'b00100000; dina <= {24'b0, wr_value, 60'b0}; end
                    3'b110: begin wea <= 8'b01000000; dina <= {12'b0, wr_value, 72'b0}; end
                    3'b111: begin wea <= 8'b10000000; dina <= {       wr_value, 84'b0}; end
                endcase
            end

            if (wr_valid_late) begin // cycle2
                case (state2)
                    WRITEA: mask_a[wr_mask_addr] <= 1'b1;
                    WRITEB: mask_b[wr_mask_addr] <= 1'b1;
                endcase
            end

            if (rd_enable) begin // cycle2
                rd_left <= 8;
            end

            if (rd_left > 0) begin // cycle0
                rd_left <= rd_left - 1;
                addrb <= {~state, rd_pos[2:0]};
            end

            if (rd_valid) begin // cycle3
                valid_out <= 1;
                column_out[11: 0] <= (state == READA) ? (mask_a[rd_pos3   ] ? doutb[11: 0] : 0) : (mask_b[rd_pos3   ] ? doutb[11: 0] : 0);
                column_out[23:12] <= (state == READA) ? (mask_a[rd_pos3+ 8] ? doutb[23:12] : 0) : (mask_b[rd_pos3+ 8] ? doutb[23:12] : 0);
                column_out[35:24] <= (state == READA) ? (mask_a[rd_pos3+16] ? doutb[35:24] : 0) : (mask_b[rd_pos3+16] ? doutb[35:24] : 0);
                column_out[47:36] <= (state == READA) ? (mask_a[rd_pos3+24] ? doutb[47:36] : 0) : (mask_b[rd_pos3+24] ? doutb[47:36] : 0);
                column_out[59:48] <= (state == READA) ? (mask_a[rd_pos3+32] ? doutb[59:48] : 0) : (mask_b[rd_pos3+32] ? doutb[59:48] : 0);
                column_out[71:60] <= (state == READA) ? (mask_a[rd_pos3+40] ? doutb[71:60] : 0) : (mask_b[rd_pos3+40] ? doutb[71:60] : 0);
                column_out[83:72] <= (state == READA) ? (mask_a[rd_pos3+48] ? doutb[83:72] : 0) : (mask_b[rd_pos3+48] ? doutb[83:72] : 0);
                column_out[95:84] <= (state == READA) ? (mask_a[rd_pos3+56] ? doutb[95:84] : 0) : (mask_b[rd_pos3+56] ? doutb[95:84] : 0);
            end else begin
                valid_out <= 0;
                column_out <= 0;
            end
         end
    end

endmodule