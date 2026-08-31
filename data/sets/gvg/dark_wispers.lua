local card = {
    api_version = 1,
    id = "GVG_041",
    name = "Dark Wispers",
    text = "<b>Choose One -</b> Summon 5 Wisps; or Give a minion +5/+5 and <b>Taunt</b>.",
    set = "GVG",
    type = "spell",
    class = "druid",
    spell_school = "nature",
    rarity = "epic",
    cost = 6,
    keywords = { "choose_one" },
}

local function has_keyword(ctx, entity, wanted)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == wanted then return true end
    end
    return false
end

local function buff_candidates(ctx, self)
    local player = ctx:controller(self)
    local result = {}
    for _, minion in ipairs(ctx:minions()) do
        local entity = ctx:entity(minion)
        local legal = not has_keyword(ctx, minion, "dormant")
        if legal and entity.controller ~= player then
            legal = not has_keyword(ctx, minion, "stealth")
                and not has_keyword(ctx, minion, "elusive")
                and not has_keyword(ctx, minion, "immune")
        end
        if legal then result[#result + 1] = minion end
    end
    return result
end

local function can_summon(ctx, self)
    return #ctx:board(ctx:controller(self)) < 7
end

card.rules = {
    can_play = function(ctx, self, current)
        return current and (can_summon(ctx, self) or #buff_candidates(ctx, self) > 0)
    end,
}

local function summon_wisps(ctx, self)
    local player = ctx:controller(self)
    for _ = 1, 5 do ctx:summon(player, "CS2_231") end
end

local function choose_buff_target(ctx, self)
    local candidates = buff_candidates(ctx, self)
    if #candidates > 0 then
        ctx:choose_entities(ctx:controller(self), "Give a minion +5/+5 and Taunt", candidates, "buff_minion")
    end
end

function card.on_choose_one(ctx, self)
    local options = {}
    if can_summon(ctx, self) then
        options[#options + 1] = { card_id = "GVG_041b", label = "Summon 5 Wisps" }
    end
    if #buff_candidates(ctx, self) > 0 then
        options[#options + 1] = { card_id = "GVG_041a", label = "Give a minion +5/+5 and Taunt" }
    end
    if #options == 1 then card.chosen(ctx, self, options[1].card_id)
    else ctx:choose_options(ctx:controller(self), "Choose One", options, "chosen") end
end

function card.chosen(ctx, self, choice)
    if choice == "GVG_041b" then summon_wisps(ctx, self)
    else choose_buff_target(ctx, self) end
end

function card.buff_minion(ctx, self, target)
    cardlib.effects.buff(ctx, target, 5, 5)
    cardlib.effects.grant_keyword(ctx, target, "taunt")
end

function card.on_choose_multiple(ctx, self)
    local candidates = buff_candidates(ctx, self)
    if #candidates > 0 then
        ctx:choose_entities(ctx:controller(self), "Give a minion +5/+5 and Taunt", candidates, "buff_minion")
    end
    -- With both choices active, the targeted buff resolves before the Wisps.
    if can_summon(ctx, self) then summon_wisps(ctx, self) end
end

card.tokens = {
    { id = "GVG_041a", spell_school = "nature", name = "Call the Guardians", text = "+5/+5 and <b>Taunt</b>.", set = "GVG", type = "spell", class = "druid", collectible = false, cost = 6 },
    { id = "GVG_041b", spell_school = "nature", name = "Nature's Defense", text = "Summon 5 Wisps.", set = "GVG", type = "spell", class = "druid", collectible = false, cost = 6 },
}

return card
