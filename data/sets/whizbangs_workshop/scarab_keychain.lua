local card = {
    api_version = 1,
    id = "TOY_006",
    name = "Scarab Keychain",
    text = "<b>Battlecry:</b> <b>Discover</b> a\n2-Cost card.",
    set = "WHIZBANGS_WORKSHOP",
    type = "minion",
    cost = 1,
    attack = 1,
    health = 1,
    tags = { "beast" },
    keywords = { "battlecry" },
}

card.on_battlecry = function(ctx, self)
    local candidates = {}
    local player = ctx:controller(self)
    local player_class = ctx:player(player).class
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.cost == 2
            and (definition.class == "neutral" or definition.class == player_class) then
            candidates[#candidates + 1] = card_id
        end
    end
    local prompt = ctx:localize(
        "Discover a 2-Cost card",
        "发现一张2费卡牌",
        "發現一張消耗為（2）的卡牌"
    )
    ctx:discover_cards(player, prompt, candidates, 3, "on_discovered")
end

card.on_discovered = function(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

return card
