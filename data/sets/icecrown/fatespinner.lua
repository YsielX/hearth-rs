local function all_minions(ctx)
    local result = {}
    for _, minion in ipairs(ctx:minions()) do result[#result + 1] = minion end
    return result
end

local function decay(ctx)
    cardlib.effects.damage_all(ctx, all_minions(ctx), 3)
end

local function growth(ctx)
    cardlib.effects.modify_all(ctx, all_minions(ctx), { attack = 2, health = 2, operation = "add" })
end

local card = {
    api_version = 1, id = "ICC_047", name = "Fatespinner",
    text = "<b>Choose a Deathrattle (Secretly) -</b> Deal 3 damage to all minions; or Give them +2/+2.",
    set = "ICECROWN", type = "minion", class = "druid", rarity = "epic",
    cost = 5, attack = 5, health = 3, tags = { "undead" },
    keywords = { "choose_one" },
}

function card.on_choose_one(ctx, self)
    ctx:choose_options(ctx:controller(self), "Choose a Deathrattle", {
        { label = "Deal 3 damage to all minions", value = 1 },
        { label = "Give all minions +2/+2", value = 2 },
    }, "fatespinner_chosen")
end

function card.fatespinner_chosen(ctx, self, choice)
    cardlib.effects.transform(ctx, self, "ICC_047t")
    ctx:set_data(self, "fatespinner_mode", choice)
end

function card.on_choose_multiple(ctx, self)
    cardlib.effects.transform(ctx, self, "ICC_047t2")
end

card.tokens = {
    {
        id = "ICC_047a", name = "Growth", text = "<b>Deathrattle:</b> Give all minions +2/+2.",
        set = "ICECROWN", type = "spell", class = "druid", collectible = false, cost = 5,
    },
    {
        id = "ICC_047b", name = "Decay", text = "<b>Deathrattle:</b> Deal 3 damage to all minions.",
        set = "ICECROWN", type = "spell", class = "druid", collectible = false, cost = 5,
    },
    {
        id = "ICC_047t", rarity = "epic", name = "Fatespinner",
        text = "<b>Secret Deathrattle:</b> Deal 3 damage to all minions; or Give them +2/+2.@<b>Secret Deathrattle:</b> Give +2/+2 to all minions.@<b>Secret Deathrattle:</b> Deal 3 damage to all minions.",
        set = "ICECROWN", type = "minion", class = "druid", collectible = false,
        cost = 5, attack = 5, health = 3, tags = { "undead" },
        triggers = {{
            event = "transformed", timing = "after", active_zones = { "board" },
            condition = function(ctx, self, event) return event.entity == self end,
            effect = function(ctx, self) ctx:grant_keyword(self, "deathrattle") end,
        }},
        on_deathrattle = function(ctx, self)
            if ctx:get_data(self, "fatespinner_mode") == 2 then growth(ctx) else decay(ctx) end
        end,
    },
    {
        id = "ICC_047t2", rarity = "epic", name = "Fatespinner",
        text = "<b>Deathrattle:</b> Deal 3 damage to all minions and give them +2/+2.",
        set = "ICECROWN", type = "minion", class = "druid", collectible = false,
        cost = 5, attack = 5, health = 3, tags = { "undead" }, keywords = { "deathrattle" },
        on_deathrattle = function(ctx)
            decay(ctx)
            ctx:continue_with("fatespinner_growth_after_decay")
        end,
        fatespinner_growth_after_decay = function(ctx) growth(ctx) end,
    },
}

return card
