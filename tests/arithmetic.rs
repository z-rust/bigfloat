use bigfloat::*;

#[test]
fn arithmetic_add() {
    let x = "123.456".to_bigfloat(256);
    let y = "789.012".to_bigfloat(256);

    let add = &x + &y;

    assert_eq!(add.to_string(50), "912.468");
}

#[test]
fn arithmetic_sub() {
    let x = "123.456".to_bigfloat(256);
    let y = "789.012".to_bigfloat(256);

    let sub = &x - &y;
    assert_eq!(sub.to_string(50), "-665.556");
}

#[test]
fn arithmetic_mul() {
    let x = "123.456".to_bigfloat(256);
    let y = "789.012".to_bigfloat(256);

    let mul = &x * &y;

    assert_eq!(mul.to_string(50), "97408.265472");
}

#[test]
fn arithmetic_div() {
    let x = "123.456".to_bigfloat(256);
    let y = "789.012".to_bigfloat(256);

    let div = &x / &y;

    assert_eq!(
        div.to_string(50),
        "0.15646910313151130781280893066265151860808200635732"
    );
}

#[test]
fn arithmetic_pi() {
    let result = const_pi(Some(50));
    println!("{}", result);
    assert_eq!(
        result,
        "3.1415926535897932384626433832795028841971693993751"
    );
}

#[test]
fn arithmetic_e() {
    let result = const_e(Some(50));
    println!("{}", result);
    assert_eq!(result, "2.7182818284590452353602874713526624977572470937");
}

#[test]
fn arithmetic_sqrt() {
    let result = sqrt(2.0, Some(50));
    println!("{}", result);
    assert_eq!(
        result,
        "1.4142135623730950488016887242096980785696718753769"
    );
}

#[test]
fn arithmetic_root() {
    let result = root(81.0, 3.0, Some(50));
    println!("{}", result);
    assert_eq!(
        result,
        "4.3267487109222251469649149323403287651756077604981"
    );
}

#[test]
fn arithmetic_pow() {
    let result = pow(
        "1.4142135623730950488016887242096980785696718753769",
        "2.0",
        Some(50),
    );
    println!("{}", result);
    assert_eq!(
        result,
        "1.9999999999999999999999999999999999999999999999999"
    );
}

#[test]
fn arithmetic_ln() {
    let result = ln(5.0, Some(50));
    println!("{}", result);
    assert_eq!(
        result,
        "1.6094379124341003746007593332261876395256013542685"
    );
}

#[test]
fn arithmetic_log2() {
    let result = log2(4.0, Some(50));
    println!("{}", result);
    assert_eq!(result, "2");
}

#[test]
fn arithmetic_log() {
    let result = log(1000.0, Some(50));
    println!("{}", result);
    assert_eq!(result, "3");
}

// #[test]
// fn arithmetic_factorial() {
//     let res = factorial("1e6", Some(500));
//     assert!(res.starts_with("8.2639316883312400623766461031726662911353"));
//     assert!(res.ends_with("e5565708"));
// }
#[test]
fn arithmetic_factorial() {
    let result = factorial("1e6", Some(500));
    println!("{}", result);
    assert_eq!(
        result,
        "8.2639316883312400623766461031726662911353
     4797896387304516777588556337961103564508444653051131146
     3973351606804210878588541464746950647836182301210975423
     2995901156417462491737988838926919341417654578323931987
     2802472198939643654445521615339205835199387989417742062
     4084159398770181880722316925205773712843685981522238931
     1521255279546829742282164292748493887784712443572285950
     9343621176452544930522658411976299056190121202414190025
     3412831943306507620700405159591511718661384475090075583
     4037427137686877042e5565708"
            .replace(" ", "")
            .replace("\n", "")
    );
}

#[test]
fn arithmetic_ln_factorial() {
    assert_eq!(
        ln_factorial(1e6, Some(50)),
        "12815518.384658169624251075892965841259873220802824"
    );
}

#[test]
fn arithmetic_rad_to_deg() {
    let result = rad_to_deg(1.0, Some(50));
    println!("{}", result);
    assert_eq!(
        result,
        "57.295779513082320876798154814105170332405472466564"
    );
}

#[test]
fn arithmetic_deg_to_rad() {
    let result = deg_to_rad(180.0, Some(50));
    println!("{}", result);
    assert_eq!(
        result,
        "3.1415926535897932384626433832795028841971693993751"
    );
}

#[test]
fn arithmetic_sin() {
    let result = sin(1.0, Some(50));
    println!("{}", result);
    assert_eq!(
        result,
        "0.84147098480789650665250232163029899962256306079837"
    );
}

#[test]
fn arithmetic_asin() {
    let result = asin(
        "0.84147098480789650665250232163029899962256306079837",
        Some(50),
    );
    println!("{}", result);
    assert_eq!(result, "1");
}

#[test]
fn arithmetic_cos() {
    let result = cos(1.0, Some(50));
    println!("{}", result);
    assert_eq!(
        result,
        "0.54030230586813971740093660744297660373231042061792"
    );
}

#[test]
fn arithmetic_acos() {
    let result = acos(
        "0.54030230586813971740093660744297660373231042061792",
        Some(50),
    );
    println!("{}", result);
    assert_eq!(result, "1");
}

#[test]
fn arithmetic_tan() {
    let result = tan(0.4, Some(50));
    println!("{}", result);
    assert_eq!(
        result,
        "0.42279321873816178815523440002094227056035703985243"
    );
}

#[test]
fn arithmetic_atan() {
    let result = atan(
        "0.42279321873816178815523440002094227056035703985243",
        Some(50),
    );
    println!("{}", result);
    assert_eq!(
        result,
        "0.4000000000000000222044604925031308084726333618164"
    );
}
