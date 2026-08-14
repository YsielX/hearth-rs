local adaptations = {
    "UNG_999t2", "UNG_999t3", "UNG_999t4", "UNG_999t5", "UNG_999t6",
    "UNG_999t7", "UNG_999t8", "UNG_999t10", "UNG_999t13", "UNG_999t14",
}

local card = {
    api_version = 1,
    id = "UNG_961",
    name = "Adaptation",
    text = "<b>Adapt</b> a friendly minion.",
    set = "UNGORO",
    type = "spell",
    class = "paladin",
    cost = 0,
    keywords = { "adapt" },
    target_mode = "required",
    targets = function(ctx, self) return ctx:friendly_minions(self) end,
}

function card.on_adapt(ctx, self, target)
    ctx:set_data(self, "adapt_target", target)
    ctx:discover_cards(
        ctx:controller(self),
        "Adapt",
        adaptations,
        3,
        "adapted"
    )
end

function card.adapted(ctx, self, adaptation)
    local target = ctx:get_data(self, "adapt_target")
    if target == nil then return end
    if adaptation == "UNG_999t2" then
        ctx:set_data(target, "living_spores", 1)
        ctx:set_data(self, "living_spores_target", target)
    elseif adaptation == "UNG_999t3" then
        ctx:buff(target, 3, 0)
    elseif adaptation == "UNG_999t4" then
        ctx:buff(target, 0, 3)
    elseif adaptation == "UNG_999t5" then
        ctx:grant_keyword(target, "elusive")
    elseif adaptation == "UNG_999t6" then
        ctx:grant_keyword(target, "taunt")
    elseif adaptation == "UNG_999t7" then
        ctx:grant_keyword(target, "windfury")
    elseif adaptation == "UNG_999t8" then
        ctx:grant_keyword(target, "divine_shield")
    elseif adaptation == "UNG_999t10" then
        ctx:grant_keyword(target, "stealth")
        ctx:set_data(self, "stealth_target", target)
    elseif adaptation == "UNG_999t13" then
        ctx:grant_keyword(target, "poisonous")
    elseif adaptation == "UNG_999t14" then
        ctx:buff(target, 1, 1)
    end
end

card.triggers = {
    {
        event = "entity_died", timing = "after", active_zones = { "graveyard" },
        condition = function(ctx, self, event)
            return ctx:get_data(self, "living_spores_target") == event.entity
        end,
        effect = function(ctx, self, event)
            local player = ctx:controller(event.entity)
            ctx:summon_at(player, "UNG_999t2t1", event.position)
            ctx:summon_at(player, "UNG_999t2t1", event.position + 1)
        end,
    },
    {
        event = "turn_started", timing = "after", active_zones = { "graveyard" },
        condition = function(ctx, self, event)
            local target = ctx:get_data(self, "stealth_target")
            return target ~= nil and target ~= 0
                and ctx:entity(target).zone == "board"
                and event.player == ctx:controller(target)
        end,
        effect = function(ctx, self, event)
            local target = ctx:get_data(self, "stealth_target")
            ctx:remove_enchantments_from(target, self)
            ctx:set_data(self, "stealth_target", 0)
        end,
    },
}

card.tokens = {
    { id = "UNG_999t2", name = "Living Spores", text = "<b>Deathrattle:</b> Summon two 1/1 Plants.", set = "UNGORO", type = "spell", class = "neutral", cost = 0 },
    { id = "UNG_999t3", name = "Flaming Claws", text = "+3 Attack", set = "UNGORO", type = "spell", class = "neutral", cost = 0 },
    { id = "UNG_999t4", name = "Rocky Carapace", text = "+3 Health", set = "UNGORO", type = "spell", class = "neutral", cost = 0 },
    { id = "UNG_999t5", name = "Liquid Membrane", text = "<b>Elusive</b>", set = "UNGORO", type = "spell", class = "neutral", cost = 0 },
    { id = "UNG_999t6", name = "Massive", text = "<b>Taunt</b>", set = "UNGORO", type = "spell", class = "neutral", cost = 0 },
    { id = "UNG_999t7", name = "Lightning Speed", text = "<b>Windfury</b>", set = "UNGORO", type = "spell", class = "neutral", cost = 0 },
    { id = "UNG_999t8", name = "Crackling Shield", text = "<b>Divine Shield</b>", set = "UNGORO", type = "spell", class = "neutral", cost = 0 },
    { id = "UNG_999t10", name = "Shrouding Mist", text = "<b>Stealth</b> until your next turn.", set = "UNGORO", type = "spell", class = "neutral", cost = 0 },
    { id = "UNG_999t13", name = "Poison Spit", text = "<b>Poisonous</b>", set = "UNGORO", type = "spell", class = "neutral", cost = 0 },
    { id = "UNG_999t14", name = "Volcanic Might", text = "+1/+1", set = "UNGORO", type = "spell", class = "neutral", cost = 0 },
    { id = "UNG_999t2t1", name = "Plant", text = "", set = "UNGORO", type = "minion", class = "neutral", cost = 1, attack = 1, health = 1 },
}

return card
