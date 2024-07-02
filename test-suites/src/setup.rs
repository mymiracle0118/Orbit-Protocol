use soroban_sdk::{log, testutils::{Address as _, Logs, MockAuth, MockAuthInvoke}, vec as svec, Address, String, Symbol, Env, Vec as SVec};

use crate::{
    dependencies::pool::{default_reserve_metadata, Request, RequestType, ReserveEmissionMetadata},
    test_fixture::{TestFixture, TokenIndex, SCALAR_7},
};

/// Create a test fixture with a pool and a whale depositing and borrowing all assets
pub fn create_fixture_with_data<'a>() -> TestFixture<'a> {

    std::println!("===================================== Fixture Create With Data ===========================================");

    let mut fixture = TestFixture::create();

    // mint whale tokens
    let frodo = fixture.users[0].clone();
    fixture.users.push(frodo.clone());
    
    fixture.tokens[TokenIndex::STABLE].mint(&frodo, &(100_000 * 10i128.pow(6)));
    fixture.tokens[TokenIndex::XLM].mint(&frodo, &(1_000_000 * SCALAR_7));
    fixture.tokens[TokenIndex::WETH].mint(&frodo, &(100 * 10i128.pow(9)));
    fixture.tokens[TokenIndex::OUSD].mint(&frodo, &(100_000 * 10i128.pow(6)));

    // mint LP tokens with whale
    // frodo has 40m BLND from drop
    fixture.tokens[TokenIndex::BLND].mint(&frodo, &(70_000_000 * SCALAR_7));
    fixture.tokens[TokenIndex::USDC].mint(&frodo, &(2_600_000 * SCALAR_7));
    fixture.lp.join_pool(
        &(10_000_000 * SCALAR_7),
        &svec![&fixture.env, 110_000_000 * SCALAR_7, 2_600_000 * SCALAR_7,],
        &frodo,
    );

    fixture.create_pool(String::from_str(&fixture.env, "Teapot"), 0_1000000, 6);

    let mut stable_config = default_reserve_metadata();
    stable_config.decimals = 6;
    stable_config.c_factor = 0_900_0000;
    stable_config.l_factor = 0_950_0000;
    stable_config.util = 0_850_0000;
    fixture.create_pool_reserve(0, TokenIndex::STABLE, &stable_config);

    let mut xlm_config = default_reserve_metadata();
    xlm_config.c_factor = 0_750_0000;
    xlm_config.l_factor = 0_750_0000;
    xlm_config.util = 0_500_0000;
    fixture.create_pool_reserve(0, TokenIndex::XLM, &xlm_config);

    let mut weth_config = default_reserve_metadata();
    weth_config.decimals = 9;
    weth_config.c_factor = 0_800_0000;
    weth_config.l_factor = 0_800_0000;
    weth_config.util = 0_700_0000;
    fixture.create_pool_reserve(0, TokenIndex::WETH, &weth_config);

    let mut ousd_config = default_reserve_metadata();
    ousd_config.decimals = 6;
    ousd_config.c_factor = 0_900_0000;
    ousd_config.l_factor = 0_950_0000;
    ousd_config.util = 0_850_0000;
    fixture.create_pool_reserve(0, TokenIndex::OUSD, &ousd_config);

    // enable emissions for pool
    let pool_fixture = &fixture.pools[0];

    let reserve_emissions: soroban_sdk::Vec<ReserveEmissionMetadata> = soroban_sdk::vec![
        &fixture.env,
        ReserveEmissionMetadata {
            res_index: 0, // STABLE
            res_type: 0,  // d_token
            share: 0_600_0000
        },
        ReserveEmissionMetadata {
            res_index: 1, // XLM
            res_type: 1,  // b_token
            share: 0_400_0000
        },
    ];
    pool_fixture.pool.set_emissions_config(&reserve_emissions);

    // deposit into backstop, add to reward zone
    fixture
        .backstop
        .deposit(&frodo, &pool_fixture.pool.address, &(50_000 * SCALAR_7));
    fixture.backstop.update_tkn_val();
    fixture
        .backstop
        .add_reward(&pool_fixture.pool.address, &Address::generate(&fixture.env));
    pool_fixture.pool.set_status(&3);
    pool_fixture.pool.update_status();

    // enable emissions
    fixture.emitter.distribute();
    fixture.backstop.gulp_emissions();
    pool_fixture.pool.gulp_emissions();

    fixture.jump(60);

    // supply and borrow OUSD for 80% utilization (close to target)
    let requests: SVec<Request> = svec![
        &fixture.env,
        Request {
            request_type: RequestType::SupplyCollateral as u32,
            address: fixture.tokens[TokenIndex::STABLE].address.clone(),
            amount: 10_000 * 10i128.pow(6),
        },
        Request {
            request_type: RequestType::Borrow as u32,
            address: fixture.tokens[TokenIndex::STABLE].address.clone(),
            amount: 8_000 * 10i128.pow(6),
        },
    ];
    pool_fixture.pool.submit(&frodo, &frodo, &frodo, &requests);
  
    // supply and borrow WETH for 50% utilization (below target)
    let requests: SVec<Request> = svec![
        &fixture.env,
        Request {
            request_type: RequestType::SupplyCollateral as u32,
            address: fixture.tokens[TokenIndex::WETH].address.clone(),
            amount: 10 * 10i128.pow(9),
        },
        Request {
            request_type: RequestType::Borrow as u32,
            address: fixture.tokens[TokenIndex::WETH].address.clone(),
            amount: 5 * 10i128.pow(9),
        },
    ];
    pool_fixture.pool.submit(&frodo, &frodo, &frodo, &requests);

    // supply and borrow XLM for 65% utilization (above target)
    let requests: SVec<Request> = svec![
        &fixture.env,
        Request {
            request_type: RequestType::SupplyCollateral as u32,
            address: fixture.tokens[TokenIndex::XLM].address.clone(),
            amount: 100_000 * SCALAR_7,
        },
        Request {
            request_type: RequestType::Borrow as u32,
            address: fixture.tokens[TokenIndex::XLM].address.clone(),
            amount: 65_000 * SCALAR_7,
        },
    ];
    pool_fixture.pool.submit(&frodo, &frodo, &frodo, &requests);

    // supply and borrow OUSD for 80% utilization (close to target)
    // let requests: SVec<Request> = svec![
    //     &fixture.env,
    //     Request {
    //         request_type: RequestType::SupplyCollateral as u32,
    //         address: fixture.tokens[TokenIndex::OUSD].address.clone(),
    //         amount: 10_000 * 10i128.pow(6),
    //     },
    //     Request {
    //         request_type: RequestType::Borrow as u32,
    //         address: fixture.tokens[TokenIndex::OUSD].address.clone(),
    //         amount: 8_000 * 10i128.pow(6),
    //     },
    // ];
   
    // pool_fixture.pool.submit(&frodo, &frodo, &frodo, &requests);

    std::println!("===================================== Fixture Create With Data Successfully ===========================================");

    fixture.jump(60 * 60); // 1 hr

    fixture.env.budget().reset_unlimited();
    fixture
}

#[cfg(test)]
mod tests {
    use crate::test_fixture::PoolFixture;

    use super::*;

    #[test]
    fn test_create_fixture_with_data_wasm() {
        let fixture: TestFixture<'_> = create_fixture_with_data();
        let frodo: &Address = fixture.users.get(0).unwrap();
        let pool_fixture: &PoolFixture = fixture.pools.get(0).unwrap();

        // validate backstop deposit and drop
        assert_eq!(
            50_000 * SCALAR_7,
            fixture.lp.balance(&fixture.backstop.address)
        );
        assert_eq!(
            10_000_000 * SCALAR_7,
            fixture.tokens[TokenIndex::BLND].balance(&fixture.admin)
        );

        // validate pool actions
        assert_eq!(
            2_000 * 10i128.pow(6),
            fixture.tokens[TokenIndex::STABLE].balance(&pool_fixture.pool.address)
        );
        // assert_eq!(
        //     2_000 * 10i128.pow(6),
        //     fixture.tokens[TokenIndex::OUSD].balance(&pool_fixture.pool.address)
        // );
        assert_eq!(
            35_000 * SCALAR_7,
            fixture.tokens[TokenIndex::XLM].balance(&pool_fixture.pool.address)
        );

        assert_eq!(
            965_000 * SCALAR_7,
            fixture.tokens[TokenIndex::XLM].balance(&frodo)
        );
        // validate emissions are turned on
        let (emis_config, emis_data) = fixture.read_reserve_emissions(0, TokenIndex::STABLE, 0);
        assert_eq!(
            emis_data.last_time,
            fixture.env.ledger().timestamp() - 60 * 61
        );
        assert_eq!(emis_data.index, 0);
        assert_eq!(0_180_0000, emis_config.eps);
        assert_eq!(
            fixture.env.ledger().timestamp() + 7 * 24 * 60 * 60 - 60 * 61,
            emis_config.expiration
        )
    }
}
