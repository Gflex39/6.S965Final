module scan_order_lut (
    input wire clk_in,
    input wire rst_in,
    input wire [5:0] x_in,
    output logic [5:0] x_out
);

    always_ff @(posedge clk_in) begin
        if (rst_in) begin
            x_out <= 6'd00;
        end else begin
            case (x_in)
                6'd00: x_out <= 6'd00;
                6'd01: x_out <= 6'd08;
                6'd02: x_out <= 6'd01;
                6'd03: x_out <= 6'd02;
                6'd04: x_out <= 6'd09;
                6'd05: x_out <= 6'd16;
                6'd06: x_out <= 6'd24;
                6'd07: x_out <= 6'd17;
                6'd08: x_out <= 6'd10;
                6'd09: x_out <= 6'd03;
                6'd10: x_out <= 6'd04;
                6'd11: x_out <= 6'd11;
                6'd12: x_out <= 6'd18;
                6'd13: x_out <= 6'd25;
                6'd14: x_out <= 6'd32;
                6'd15: x_out <= 6'd40;
                6'd16: x_out <= 6'd33;
                6'd17: x_out <= 6'd26;
                6'd18: x_out <= 6'd19;
                6'd19: x_out <= 6'd12;
                6'd20: x_out <= 6'd05;
                6'd21: x_out <= 6'd06;
                6'd22: x_out <= 6'd13;
                6'd23: x_out <= 6'd20;
                6'd24: x_out <= 6'd27;
                6'd25: x_out <= 6'd34;
                6'd26: x_out <= 6'd41;
                6'd27: x_out <= 6'd48;
                6'd28: x_out <= 6'd56;
                6'd29: x_out <= 6'd49;
                6'd30: x_out <= 6'd42;
                6'd31: x_out <= 6'd35;
                6'd32: x_out <= 6'd28;
                6'd33: x_out <= 6'd21;
                6'd34: x_out <= 6'd14;
                6'd35: x_out <= 6'd07;
                6'd36: x_out <= 6'd15;
                6'd37: x_out <= 6'd22;
                6'd38: x_out <= 6'd29;
                6'd39: x_out <= 6'd36;
                6'd40: x_out <= 6'd43;
                6'd41: x_out <= 6'd50;
                6'd42: x_out <= 6'd57;
                6'd43: x_out <= 6'd58;
                6'd44: x_out <= 6'd51;
                6'd45: x_out <= 6'd44;
                6'd46: x_out <= 6'd37;
                6'd47: x_out <= 6'd30;
                6'd48: x_out <= 6'd23;
                6'd49: x_out <= 6'd31;
                6'd50: x_out <= 6'd38;
                6'd51: x_out <= 6'd45;
                6'd52: x_out <= 6'd52;
                6'd53: x_out <= 6'd59;
                6'd54: x_out <= 6'd60;
                6'd55: x_out <= 6'd53;
                6'd56: x_out <= 6'd46;
                6'd57: x_out <= 6'd39;
                6'd58: x_out <= 6'd47;
                6'd59: x_out <= 6'd54;
                6'd60: x_out <= 6'd61;
                6'd61: x_out <= 6'd62;
                6'd62: x_out <= 6'd55;
                6'd63: x_out <= 6'd63;
            endcase
        end
    end

endmodule