local totems = { "NEW1_009", "CS2_050", "CS2_051", "CS2_052" }

local power = {
    api_version = 1,
    module_type = "hero_power",
    id = "HERO_02bp",
    name = "Totemic Call",
    text = "<b>Hero Power</b>\nSummon a random basic Totem.",
    set = "LEGACY",
    class = "shaman",
    cost = 2,
}

function power.on_play(ctx, self)
    local player = ctx:controller(self)
    local present = {}
    for _, entity in ipairs(ctx:board(player)) do
        present[ctx:entity(entity).card_id] = true
    end
    local candidates = {}
    for _, card_id in ipairs(totems) do
        if not present[card_id] then candidates[#candidates + 1] = card_id end
    end
    if #candidates > 0 then ctx:random_value(candidates, "summon_totem") end
end

function power.summon_totem(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

power.tokens = {
    {
        id = "NEW1_009", name = "Healing Totem",
        text = "At the end of your turn, restore #1 Health to all friendly minions.",
        set = "LEGACY", type = "minion", class = "shaman",
        cost = 1, attack = 0, health = 2, tags = { "totem" },
        triggers = {
            {
                event = "turn_ended", timing = "after", active_zones = { "board" },
                condition = function(ctx, self, event)
                    return event.player == ctx:controller(self)
                end,
                effect = function(ctx, self)
                    for _, minion in ipairs(ctx:friendly_minions(self)) do
                        cardlib.effects.heal(ctx, minion, 1)
                    end
                end,
            },
        },
    },
    {
        id = "CS2_050", name = "Searing Totem", text = "",
        set = "LEGACY", type = "minion", class = "shaman",
        cost = 1, attack = 1, health = 1, tags = { "totem" },
    },
    {
        id = "CS2_051", name = "Stoneclaw Totem", text = "<b>Taunt</b>",
        set = "LEGACY", type = "minion", class = "shaman",
        cost = 1, attack = 0, health = 2, tags = { "totem" }, keywords = { "taunt" },
    },
    {
        id = "CS2_052", name = "Wrath of Air Totem", text = "<b>Spell Damage +1</b>",
        set = "LEGACY", type = "minion", class = "shaman",
        cost = 1, attack = 0, health = 2, tags = { "totem" },
        keywords = { "spell_damage" }, keyword_params = { spell_damage = 1 },
    },
}

return power
