local adaptations = {
    "UNG_999t2", "UNG_999t3", "UNG_999t4", "UNG_999t5", "UNG_999t6",
    "UNG_999t7", "UNG_999t8", "UNG_999t10", "UNG_999t13", "UNG_999t14",
}
local card = {
    api_version = 1, id = "UNG_089", name = "Gentle Megasaur",
    text = "<b>Battlecry:</b> <b>Adapt</b> your Murlocs.",
    set = "UNGORO", type = "minion", rarity = "epic", cost = 4, attack = 5, health = 4,
    tags = { "beast" }, keywords = { "battlecry" },
}
local function murloc(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == "murloc" or tag == "all" then return true end
    end
    return false
end
local function apply_adaptation(ctx, target, adaptation)
    if ctx:entity(target).zone ~= "board" then return end
    for _, keyword in ipairs(ctx:entity(target).keywords) do if keyword == "dormant" then return end end
    if adaptation == "UNG_999t2" then
        ctx:attach_hook(target, "on_deathrattle", "UNG_999t2")
        cardlib.effects.grant_keyword(ctx, target, "deathrattle")
    elseif adaptation == "UNG_999t3" then
        cardlib.effects.buff(ctx, target, 3, 0)
    elseif adaptation == "UNG_999t4" then
        cardlib.effects.buff(ctx, target, 0, 3)
    elseif adaptation == "UNG_999t5" then
        cardlib.effects.grant_keyword(ctx, target, "elusive")
    elseif adaptation == "UNG_999t6" then
        cardlib.effects.grant_keyword(ctx, target, "taunt")
    elseif adaptation == "UNG_999t7" then
        cardlib.effects.grant_keyword(ctx, target, "windfury")
    elseif adaptation == "UNG_999t8" then
        cardlib.effects.grant_keyword(ctx, target, "divine_shield")
    elseif adaptation == "UNG_999t10" then
        ctx:grant_keyword_until_next_turn(target, "stealth")
    elseif adaptation == "UNG_999t13" then
        cardlib.effects.grant_keyword(ctx, target, "poisonous")
    elseif adaptation == "UNG_999t14" then
        cardlib.effects.buff(ctx, target, 1, 1)
    end
end
function card.on_battlecry(ctx, self)
    local targets = {}
    for _, entity in ipairs(ctx:friendly_minions(self)) do if murloc(ctx, entity) then targets[#targets + 1] = entity end end
    if #targets == 0 then return end
    for index, entity in ipairs(targets) do ctx:set_data(self, "adapt_target_" .. index, entity) end
    ctx:set_data(self, "adapt_target_count", #targets)
    ctx:discover_cards(ctx:controller(self), "Adapt your Murlocs", adaptations, 3, "adapt_megasaur_murlocs")
end
function card.adapt_megasaur_murlocs(ctx, self, adaptation)
    local player = ctx:controller(self)
    for index = 1, ctx:get_data(self, "adapt_target_count") do
        local target = ctx:get_data(self, "adapt_target_" .. index)
        local entity = ctx:entity(target)
        if entity.zone == "board" and entity.controller == player and murloc(ctx, target) then
            apply_adaptation(ctx, target, adaptation)
        end
    end
end
return card
