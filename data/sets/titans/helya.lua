local card = {
    api_version = 1,
    id = "TTN_850",
    name = "Helya",
    text = "[x]<b>Battlecry:</b> Shuffle all three\nPlagues into your opponent's\ndeck. Plagues they draw this\ngame are unending.",
    set = "TITANS",
    type = "minion",
    class = "death_knight",
    rarity = "legendary",
    cost = 4,
    attack = 4,
    health = 4,
    rune_cost = { unholy = 1 },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local opponent = ctx:opponent(player)
    for _, plague in ipairs(cardlib.plagues.ids) do
        cardlib.plagues.shuffle(ctx, player, opponent, plague)
    end
    ctx:grant_public_player_keyword(opponent, "unending_plagues")
end

local function plague_damage(ctx, self, card_id)
    local player = ctx:controller(self)
    cardlib.plagues.reshuffle_if_unending(ctx, player, card_id)
    cardlib.effects.damage_ignoring_spell_damage(ctx, ctx:player(player).hero, 2)
    return player
end

card.tokens = {
    {
        id = "TTN_450t",
        name = "Blood Plague",
        text = "<b>Casts When Drawn</b>\nTake 2 damage.\nRestore 2 Health\nto the enemy hero.",
        set = "TITANS",
        type = "spell",
        class = "death_knight",
        collectible = false,
        cost = 1,
        keywords = { "casts_when_drawn" },
        on_play = function(ctx, self)
            local player = plague_damage(ctx, self, "TTN_450t")
            cardlib.effects.heal(ctx, ctx:player(ctx:opponent(player)).hero, 2)
        end,
    },
    {
        id = "TTN_450t2",
        name = "Unholy Plague",
        text = "<b>Casts When Drawn</b>\nTake 2 damage.\nSummon a 2/2 Undead for your opponent.",
        set = "TITANS",
        type = "spell",
        class = "death_knight",
        collectible = false,
        cost = 1,
        keywords = { "casts_when_drawn" },
        on_play = function(ctx, self)
            local player = plague_damage(ctx, self, "TTN_450t2")
            ctx:summon(ctx:opponent(player), "RLK_070t")
        end,
    },
    {
        id = "TTN_450t3",
        name = "Frost Plague",
        text = "[x]<b>Casts When Drawn</b>\nTake 2 damage.\nYour next card costs\n(1) more <i>(up to 10)</i>.",
        set = "TITANS",
        type = "spell",
        class = "death_knight",
        collectible = false,
        cost = 1,
        keywords = { "casts_when_drawn" },
        on_play = function(ctx, self)
            local player = plague_damage(ctx, self, "TTN_450t3")
            ctx:increment_player_data(player, "frost_plague_surcharge", 1)
            ctx:grant_player_keyword(player, "frost_plague_surcharge")
        end,
    },
    {
        id = "RLK_070t",
        name = "Undead Peasant",
        text = "",
        set = "RETURN_OF_THE_LICH_KING",
        type = "minion",
        class = "neutral",
        collectible = false,
        cost = 2,
        attack = 2,
        health = 2,
        tags = { "undead" },
    },
}

return card
