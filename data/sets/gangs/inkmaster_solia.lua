local KEY = "next_spell_cost_zero_this_turn"

local function no_duplicates(ctx, player)
    local seen = {}
    for _, entity in ipairs(ctx:deck(player)) do
        local id = ctx:entity(entity).card_id
        if seen[id] then return false end
        seen[id] = true
    end
    return true
end

return {
    api_version = 1,
    id = "CFM_687",
    name = "Inkmaster Solia",
    text = "[x]<b>Battlecry:</b> If your deck has\nno duplicates, the next\nspell you cast this turn\ncosts (0).",
    set = "GANGS",
    type = "minion",
    class = "mage",
    rarity = "legendary",
    cost = 7,
    attack = 5,
    health = 5,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        local player = ctx:controller(self)
        if no_duplicates(ctx, player) then
            ctx:set_player_data(player, KEY, 1)
            ctx:grant_player_keyword(player, KEY)
        end
    end,
}
