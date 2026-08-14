local card = {
    api_version = 1, id = "ICC_054", name = "Spreading Plague",
    text = "Summon a 1/5 Scarab with <b>Taunt</b>. If your opponent has more minions, cast this again.",
    set = "ICECROWN", type = "spell", class = "druid", rarity = "rare",
    spell_school = "nature", cost = 6,
}

local function minion_count(ctx, player)
    local count = 0
    for _, minion in ipairs(ctx:minions()) do
        if ctx:controller(minion) == player then count = count + 1 end
    end
    return count
end

function card.on_play(ctx, self)
    local player = ctx:controller(self)
    if #ctx:board(player) >= 7 then return end
    ctx:summon(player, "ICC_832t4")
    ctx:continue_with("spreading_plague_continue")
end

function card.spreading_plague_continue(ctx, self)
    local player = ctx:controller(self)
    if #ctx:board(player) < 7 and minion_count(ctx, ctx:opponent(player)) > minion_count(ctx, player) then
        ctx:cast_spell(player, "ICC_054")
    end
end

return card
