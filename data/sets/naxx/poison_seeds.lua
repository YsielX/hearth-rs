local card = {
    api_version = 1,
    id = "FP1_019",
    name = "Poison Seeds",
    text = "Destroy all minions and summon 2/2 Treants to replace them.",
    set = "NAXX",
    type = "spell",
    class = "druid",
    rarity = "common",
    spell_school = "nature",
    cost = 4,
    tokens = {
        {
            id = "FP1_019t",
            name = "Treant",
            text = "",
            set = "NAXX",
            type = "minion",
            class = "druid",
            cost = 1,
            attack = 2,
            health = 2,
        },
    },
}

function card.on_play(ctx, self)
    local minions = {}
    local replacements = {}
    for player = 0, 1 do
        for _, entity in ipairs(ctx:board(player)) do
            if ctx:entity(entity).type == "minion" then
                minions[#minions + 1] = entity
                replacements[#replacements + 1] = {
                    player = player,
                    position = ctx:board_position(entity),
                }
            end
        end
    end
    cardlib.effects.destroy_all(ctx, minions)
    ctx:continue_with_value("summon_replacements", replacements)
end

function card.summon_replacements(ctx, self, replacements)
    for _, replacement in ipairs(replacements) do
        ctx:summon_at(replacement.player, "FP1_019t", replacement.position)
    end
end

return card
