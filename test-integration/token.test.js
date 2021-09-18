describe('Pool Token', function () {
  let user_A, user_B, user_C

  jasmine.DEFAULT_TIMEOUT_INTERVAL = 1200000;

  beforeAll(async function () {
    // NOTE: USER_A is the OWNER
    user_A = 'test-account-1629671080959-5153795' 
    user_B = 'test-account-1629670943068-5873996'
    user_C = 'test-account-1629670799126-8779043'

    pool_party_contract = 'test-account-1629845638745-7027135'
    token_contract = nearConfig.contractName

    const near = await nearlib.connect(nearConfig);

    function create_contract_pool(user){
      return near.loadContract(pool_party_contract, {
        viewMethods: ['get_account', 'get_pool_info'],
        changeMethods: ['deposit_and_stake', 'raffle'],
        sender: user
      })
    }

    function create_contract_token(user){
      return near.loadContract(token_contract, {
        viewMethods: ['ft_balance_of'],
        changeMethods: ['new',
                        'cache_pool_party_reserve',
                        'cache_pool_party_reserve_callback',
                        'exchange_tokens_for_tickets', 
                        'exchange_tokens_for_tickets_callback',
                        'exchange_near_for_tokens',
                        'exchange_near_for_tokens_callback',
                        'storage_deposit', 'ft_transfer'],
        sender: user
      })
    }

    pool_A = await create_contract_pool(user_A)
    pool_B = await create_contract_pool(user_B)
    pool_C = await create_contract_pool(user_C)

    token_A = await create_contract_token(user_A)
    token_B = await create_contract_token(user_B)
    token_C = await create_contract_token(user_C)

    shared_var = 0
    original = 0

    exchange_near_for_tokens = async function(amount, contract){
      amount = nearAPI.utils.format.parseNearAmount(amount.toString())
      return await contract.account.functionCall(
        token_contract, 'exchange_near_for_tokens', {}, 300000000000000, amount
      )
    }

    exchange_tokens_for_tickets = async function(amount_token, contract){
      // amount = nearAPI.utils.format.parseNearAmount(amount_token.toString())
      return await contract.account.functionCall(
        token_contract, 'exchange_tokens_for_tickets',
        {amount_tokens: amount_token}, 300000000000000, 0
      )
    }

    cache_pool_party_reserve = async function(){
      // amount = nearAPI.utils.format.parseNearAmount(amount_token.toString())
      return await token_A.account.functionCall(
        token_contract, 'cache_pool_party_reserve',
        {}, 300000000000000, 0
      )
    }

    storage_deposit = async function(contract){
      amount = nearAPI.utils.format.parseNearAmount("0.00125")
      return await contract.account.functionCall(
        token_contract, 'storage_deposit', {}, 300000000000000, amount
      )
    }

    ft_transfer = async function(contract, to, token_amount){
      return await contract.account.functionCall(
        token_contract, 'ft_transfer', 
        {receiver_id: to, amount: token_amount},
        300000000000000, 1
      )
    }

    ft_balance_of = async function(whom){
      let balance = await token_A.ft_balance_of({account_id: whom})
      return balance
    }

    // POOL PARTY

    deposit_and_stake = async function(amount, contract){
      amount = nearAPI.utils.format.parseNearAmount(amount.toString())
      return await contract.account.functionCall(
        pool_party_contract, 'deposit_and_stake', {}, 300000000000000, amount
      )
    }

    raffle = async function(contract=pool_A){
      return await contract.account.functionCall(
        pool_party_contract, 'raffle', {}, 300000000000000, 0
      )
    }

    get_account = async function(account_id, contract=pool_A){
      let info = await contract.get_account({account_id})
      info.staked_balance = parseFloat(nearlib.utils.format.formatNearAmount(info.staked_balance))
      info.unstaked_balance = parseFloat(nearlib.utils.format.formatNearAmount(info.unstaked_balance))
      info.available_when = Number(info.available_when)
      return info
    }

    get_pool_info = async function(contract){
      let result = await contract.account.functionCall(
        pool_party_contract, 'get_pool_info', {}, 300000000000000, 0
      )
      info = nearlib.providers.getTransactionLastResult(result)
      info.total_staked = parseFloat(nearAPI.utils.format.formatNearAmount(info.total_staked))
      info.prize = parseFloat(nearAPI.utils.format.formatNearAmount(info.prize))
      info.reserve = parseFloat(nearAPI.utils.format.formatNearAmount(info.reserve))
      return info  
    }

  });

  describe('TOKENs', function () {
    it("starts", async function(){
      await token_A.new()
      await cache_pool_party_reserve()
      await raffle()
    })

    // Test that nothing can be done if the cache is not updated

    it("lets users register", async function(){
      await storage_deposit(token_B);
      await storage_deposit(token_C);
    })

    it("transfer tokens to user", async function(){
      await ft_transfer(token_A, user_B, "1000000")
      await ft_transfer(token_A, user_C, "1000000")

      let balance_B = await ft_balance_of(user_B)
      let balance_C = await ft_balance_of(user_C)
      expect(balance_B).toBe("1000000")
      expect(balance_C).toBe("1000000")
    })

    it("pool party has a prize", async function(){
      let info = await get_pool_info(pool_A)
      console.log(info)
      console.log("The reserve of pool party:", info.reserve)
      expect(info.reserve > 0).toBe(true)

      await deposit_and_stake(info.reserve, pool_A)

      let account_A = await get_account(user_A)
      console.log(account_A)

      expect(account_A.staked_balance).toBe(info.reserve)
    })

    it("the users can deposit in pool party", async function(){
      await deposit_and_stake(1, pool_B)
      await deposit_and_stake(1, pool_C)
    })


    it("changes tokens to tickets", async function(){
      let accountB = await get_account(user_B)
      let pool_info = await get_pool_info(pool_A)

      await exchange_tokens_for_tickets("100000", token_B)

      let updatedB = await get_account(user_B)
      let updated_info = await get_pool_info(pool_A)

      expect(updated_info.reserve).toBeCloseTo(pool_info.reserve * 0.99)
      expect(updatedB.staked_balance).toBeCloseTo(accountB.staked_balance + pool_info.reserve * 0.01)
      shared_var = pool_info.reserve * 0.01
      original = pool_info.reserve
      console.log("1% of reserve", shared_var)

      let balance_B = await ft_balance_of(user_B)
      expect(balance_B).toBe("900000")

      let balance_contract = await ft_balance_of(nearConfig.contractName)
      expect(balance_contract).toBe("100000")
    })

    it("changes NEAR to tickets", async function(){
      let balance_owner = await get_account(token_contract)

      await exchange_near_for_tokens(shared_var, token_B)

      let updated_B = await ft_balance_of(user_B)
      let balance_updated = await get_account(token_contract)

      expect(balance_updated.staked_balance).toBeCloseTo(balance_owner.staked_balance + shared_var)
      expect(parseInt(updated_B) >= 999999).toBe(true)
      expect(parseInt(updated_B) <= 1000000).toBe(true)
    })

  });
});