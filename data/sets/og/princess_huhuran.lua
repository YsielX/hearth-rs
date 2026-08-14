local function deathrattle(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "deathrattle" then return true end
    end
    return false
end
local card = {
    api_version = 1, id = "OG_309", name = "Princess Huhuran",
    text = "<b>Battlecry:</b> Trigger a friendly minion's <b>Deathrattle</b>.",
    set = "OG", type = "minion", class = "hunter", rarity = "legendary",
    cost = 5, attack = 6, health = 5, tags = { "beast" }, keywords = { "battlecry" },
    target_mode = "required_if_available",
}
function card.targets(ctx, self)
    local result = {}
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        if minion ~= self and deathrattle(ctx, minion) then result[#result + 1] = minion end
    end
    return result
end
function card.on_battlecry(ctx, self, target)
    if not target then return end
    local repetitions = 1
    for _, keyword in ipairs(ctx:entity(target).keywords) do
        if keyword == "deathrattle_repeater" then repetitions = 2 break end
    end
    for _ = 1, repetitions do
        ctx:trigger_hook(target, "on_deathrattle", ctx:board_position(target))
    end
end
return card
