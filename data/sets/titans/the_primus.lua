local card = {
    api_version = 1,
    id = "TTN_737",
    name = "The Primus",
    text = "<b>Titan</b>\nAfter this uses an ability, <b>Discover</b> a card with that Rune.",
    set = "TITANS",
    type = "minion",
    class = "death_knight",
    rarity = "legendary",
    cost = 8,
    attack = 7,
    health = 9,
    keywords = { "titan" },
}

local function discover_rune_card(ctx, self, rune)
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.class == "death_knight" and definition.rune_cost[rune] > 0 then
            pool[#pool + 1] = card_id
        end
    end
    if #pool == 0 then return end
    local prompts = {
        blood = { "Discover a Blood Rune card", "发现一张鲜血符文牌", "發現一張血魄符文牌" },
        frost = { "Discover a Frost Rune card", "发现一张冰霜符文牌", "發現一張冰霜符文牌" },
        unholy = { "Discover an Unholy Rune card", "发现一张邪恶符文牌", "發現一張穢邪符文牌" },
    }
    ctx:discover_cards(
        ctx:controller(self),
        ctx:localize(prompts[rune][1], prompts[rune][2], prompts[rune][3]),
        pool,
        3,
        "receive_rune_card"
    )
end

card.action_target_modes = { titan_1 = "required" }
card.action_semantic_cards = {
    titan_1 = "TTN_737t",
    titan_2 = "TTN_737t1",
    titan_3 = "TTN_737t3",
}
card.action_targets = {
    titan_1 = function(ctx, self) return ctx:enemy_minions(self) end,
}

card.action_effects = {
    titan_1 = function(ctx, self, spent, target)
        local health = ctx:entity(target).health
        cardlib.effects.destroy(ctx, target)
        cardlib.effects.buff(ctx, self, 0, health)
        cardlib.effects.buff(ctx, ctx:player(ctx:controller(self)).hero, 0, health)
        discover_rune_card(ctx, self, "blood")
    end,
    titan_2 = function(ctx, self)
        local player = ctx:controller(self)
        ctx:summon(player, "TTN_737t2")
        ctx:summon(player, "TTN_737t2")
        discover_rune_card(ctx, self, "unholy")
    end,
    titan_3 = function(ctx, self)
        local player = ctx:controller(self)
        ctx:increment_player_data(player, "primus_frost_runes", 1)
        ctx:grant_player_keyword(player, "primus_frost_runes")
        discover_rune_card(ctx, self, "frost")
    end,
}

function card.receive_rune_card(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
end

card.tokens = {
    {
        id = "TTN_737t",
        name = "Runes of Blood",
        text = "Destroy an enemy minion. This minion and your hero gain its Health.",
        set = "TITANS",
        type = "spell",
        class = "death_knight",
        collectible = false,
        cost = 0,
        rune_cost = { blood = 1 },
    },
    {
        id = "TTN_737t1",
        name = "Runes of the Unholy",
        text = "Summon two 3/3 Undead with <b>Taunt</b>\nand <b>Reborn</b>.",
        set = "TITANS",
        type = "spell",
        class = "death_knight",
        collectible = false,
        cost = 0,
        rune_cost = { unholy = 1 },
    },
    {
        id = "TTN_737t2",
        name = "Servant of the Primus",
        text = "<b>Taunt</b>\n<b>Reborn</b>",
        set = "TITANS",
        type = "minion",
        class = "death_knight",
        collectible = false,
        cost = 3,
        attack = 3,
        health = 3,
        tags = { "undead" },
        keywords = { "taunt", "reborn" },
    },
    {
        id = "TTN_737t3",
        name = "Runes of Frost",
        text = "The next spell you cast\ncosts (3) less and has <b>Spell Damage +3</b>.",
        set = "TITANS",
        type = "spell",
        class = "death_knight",
        collectible = false,
        cost = 0,
        rune_cost = { frost = 1 },
    },
}

return card
