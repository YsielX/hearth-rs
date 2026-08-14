local card = {
    api_version = 1,
    id = "BAR_065",
    name = "Venomous Scorpid",
    text = "<b>Poisonous</b>\n<b>Battlecry:</b> <b>Discover</b> a spell.",
    set = "THE_BARRENS",
    type = "minion",
    cost = 3,
    attack = 1,
    health = 3,
    tags = { "beast" },
    keywords = { "poisonous", "battlecry" },
}

card.on_battlecry = function(ctx, self)
    local candidates = {}
    local player = ctx:controller(self)
    local player_class = ctx:player(player).class
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "spell"
            and (definition.class == "neutral" or definition.class == player_class) then
            candidates[#candidates + 1] = card_id
        end
    end
    local prompt = ctx:localize(
        "Discover a spell",
        "发现一张法术牌",
        "發現一張法術牌"
    )
    ctx:discover_cards(player, prompt, candidates, 3, "on_discovered")
end

card.on_discovered = function(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

return card
