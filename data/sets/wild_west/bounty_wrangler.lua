local function give_coin_once(ctx, self)
    if ctx:get_data(self, "coin_given") ~= 0 then return end
    ctx:set_data(self, "coin_given", 1)
    cardlib.effects.give_card(ctx, ctx:controller(self), "GAME_005")
end

return {
    api_version = 1,
    id = "WW_363", rarity = "rare",
    name = "Bounty Wrangler",
    text = "<b>Quickdraw or Combo:</b>\nGet a Coin.",
    set = "WILD_WEST",
    type = "minion",
    class = "rogue",
    cost = 3,
    attack = 3,
    health = 4,
    keywords = { "quickdraw", "combo" },
    on_quickdraw = give_coin_once,
    on_combo = give_coin_once,
}
