local function has_tribe(ctx, entity, tribe)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == tribe or tag == "all" then return true end
    end
    return false
end

local function friendly_tribe(ctx, self, tribe)
    local result = {}
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        local dormant = false
        for _, keyword in ipairs(ctx:entity(minion).keywords) do
            if keyword == "dormant" then dormant = true break end
        end
        if not dormant and has_tribe(ctx, minion, tribe) then result[#result + 1] = minion end
    end
    return result
end

local card = {
    api_version = 1,
    id = "KAR_702",
    name = "Menagerie Magician",
    text = "<b>Battlecry:</b> Give a random friendly Beast, Dragon, and Murloc +2/+2.",
    set = "KARA",
    type = "minion",
    rarity = "common",
    cost = 5,
    attack = 4,
    health = 4,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local beasts = friendly_tribe(ctx, self, "beast")
    local dragons = friendly_tribe(ctx, self, "dragon")
    local murlocs = friendly_tribe(ctx, self, "murloc")
    if #beasts > 0 then ctx:random_entity(beasts, "buff_beast") end
    if #dragons > 0 then ctx:random_entity(dragons, "buff_dragon") end
    if #murlocs > 0 then ctx:random_entity(murlocs, "buff_murloc") end
end

function card.buff_beast(ctx, self, target) cardlib.effects.buff(ctx, target, 2, 2) end
function card.buff_dragon(ctx, self, target) cardlib.effects.buff(ctx, target, 2, 2) end
function card.buff_murloc(ctx, self, target) cardlib.effects.buff(ctx, target, 2, 2) end

return card
