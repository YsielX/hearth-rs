local random_spell = {
    api_version = 1,
    module_type = "library",
    id = "random_spell",
}

function random_spell.choose(ctx, player, candidates, count, resume_hook)
    if count <= 0 or #candidates == 0 then return end
    local options = {}
    for _, card_id in ipairs(candidates) do
        options[#options + 1] = {
            player = player,
            card_id = card_id,
            remaining = count - 1,
        }
    end
    ctx:random_value(options, resume_hook)
end

function random_spell.cast(ctx, choice)
    ctx:cast_spell(choice.player, choice.card_id, {
        skip_if_invalid = true,
        random_target = true,
        choice_policy = "random",
    })
end

return random_spell
