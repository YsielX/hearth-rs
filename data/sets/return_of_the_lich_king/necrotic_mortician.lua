local function is_undead(definition)
    for _, tag in ipairs(definition.tags or {}) do
        if tag == "undead" or tag == "all" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "RLK_116",
    name = "Necrotic Mortician",
    text = "<b>Battlecry:</b> If a friendly Undead died after your\nlast turn, <b>Discover</b> an Unholy Rune card.",
    set = "RETURN_OF_THE_LICH_KING",
    type = "minion",
    class = "death_knight",
    rarity = "common",
    cost = 2,
    attack = 2,
    health = 3,
    rune_cost = { unholy = 1 },
    keywords = { "battlecry" },
}

local function undead_died_after_last_turn(ctx, player)
    local threshold = math.max(1, ctx:turn() - 1)
    for _, record in ipairs(ctx:minion_death_records(player)) do
        if record.turn >= threshold and is_undead(ctx:card_definition(record.card_id)) then
            return true
        end
    end
    return false
end

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    if not undead_died_after_last_turn(ctx, player) then return end
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.class == "death_knight" and definition.rune_cost.unholy > 0 then
            pool[#pool + 1] = card_id
        end
    end
    if #pool > 0 then
        ctx:discover_cards(
            player,
            ctx:localize(
                "Discover an Unholy Rune card",
                "发现一张邪恶符文牌",
                "發現一張穢邪符文牌"
            ),
            pool,
            3,
            "receive_unholy_card"
        )
    end
end

function card.receive_unholy_card(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

return card
